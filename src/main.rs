// OpenKeg, the lightweight backend of the Musikverein Leopoldsdorf.
// Copyright (C) 2022  Richard Stöckl
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

#[macro_use]
extern crate rocket;

use std::env;
use std::sync::Arc;

use figment::Figment;
use ldap3::tokio::task;
use okapi::merge::merge_specs;
use okapi::openapi3::OpenApi;
use reqwest::Client;
use rocket::config::Ident;
use rocket::fairing::AdHoc;
use rocket::fs::{FileServer, Options};
use rocket::tokio::sync::RwLock;
use rocket::{Build, Rocket};
use rocket_okapi::mount_endpoints_and_merged_docs;
use rocket_okapi::settings::OpenApiSettings;

use crate::config::Config;
use crate::cors::Cors;
use crate::database::initialize_client;
use crate::info::{get_info_routes_and_docs, ServerInfo};
use crate::ldap::auth;
use crate::ldap::sync::member_synchronization_task;
use crate::members::state::MemberState;
use crate::user::key::{read_private_key, read_public_key};

mod api_result;
mod archive;
mod config;
mod cors;
mod database;
mod info;
mod ldap;
mod members;
mod schema_util;
mod user;

pub type MemberStateMutex = Arc<RwLock<MemberState>>;
pub type DatabaseClient = Client;

#[rocket::main]
async fn main() {
    env_logger::init();
    info!(
        "Welcome to OpenKeg {} - The backend of the Musikverein Leopoldsdorf!",
        env!("CARGO_PKG_VERSION")
    );
    let figment = config::read_config().merge((
        "ident",
        Ident::try_new(keg_user_agent()).expect("Valid ident for server response"),
    ));
    let member_state = MemberState::mutex();
    let mut server_result = create_server(figment).manage(member_state);
    server_result = manage_keys(server_result).attach(Cors);
    let config = server_result.figment().extract::<Config>().expect("config");
    server_result = server_result.manage(initialize_client(&config).await);
    server_result = server_result.manage(ServerInfo::new());
    server_result = configure_static_directory(server_result);
    register_user_sync_task(&server_result);
    match server_result.launch().await {
        Ok(_) => info!("Shutdown OpenKeg!"),
        Err(err) => error!("failed to start: {}", err.to_string()),
    }
}

/// Generate the [String] used for identifying the server software through the network such as HTTP.
/// The format will be `{PKG_NAME}/{PKG_VERSION} ({OS} {ARCH})`.
/// An example would be `openkeg/1.3 (linux risc64)`.
///
/// returns: String which contains the version
pub fn keg_user_agent() -> String {
    format!(
        "{}/{} ({} {})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH
    )
}

fn create_server(figment: Figment) -> Rocket<Build> {
    let openapi_settings = openapi_settings();
    let (info_route, info_spec) = get_info_routes_and_docs(&openapi_settings);
    let mut rocket = rocket::custom(figment).attach(AdHoc::config::<Config>());
    let mut openapi_spec_header = custom_openapi_spec(&rocket);
    merge_specs(&mut openapi_spec_header, &"".to_string(), &info_spec)
        .expect("OpenApi spec and routes");
    let custom_spec = (info_route, openapi_spec_header);
    mount_endpoints_and_merged_docs! {
        rocket, "/api/v1".to_owned(), openapi_settings,
        "" => custom_spec,
        "/scores" => archive::get_scores_routes_and_docs(&openapi_settings),
        "/books" => archive::get_books_routes_and_docs(&openapi_settings),
        "/statistics" => archive::get_statistics_routes_and_docs(&openapi_settings),
        "/members" => members::get_routes_and_docs(&openapi_settings),
        "/user" => user::get_routes_and_docs(&openapi_settings),
    };
    rocket.mount("/", get_info_routes_and_docs(&openapi_settings).0.to_vec())
}

fn register_user_sync_task(server: &Rocket<Build>) {
    let config: Config = server.figment().extract().expect("config");
    let member_state_option = server.state::<MemberStateMutex>();
    if member_state_option.is_none() {
        warn!("Unable to retrieve member state, scheduled user synchronization will not work");
        return;
    }
    let mut member_state_clone = member_state_option
        .expect("Member state for synchronizing")
        .clone();
    task::spawn(async move {
        member_synchronization_task(&config, &mut member_state_clone).await;
    });
}

/// Let the server manage the private and the public key.
/// Warnings will be printed to the log if this operation fails.
///
/// # Arguments
///
/// * `server`: the server where to register the keys
///
/// returns: Rocket<Build>
fn manage_keys(server: Rocket<Build>) -> Rocket<Build> {
    let config: Config = server.figment().extract().expect("config");
    info!("Read the public and the private key");
    let mut server_manage = server;
    match read_private_key(&config) {
        Ok(private_key) => {
            server_manage = server_manage.manage(private_key);
            info!("Private key successfully added to application state");
        }
        Err(err) => warn!(
            "Unable to read the private key from {}: {}",
            config.cert.private_key_path, err
        ),
    }
    match read_public_key(&config) {
        Ok(public_key) => {
            server_manage = server_manage.manage(public_key);
            info!("Public key successfully added to application state");
        }
        Err(err) => warn!(
            "Unable to read the public key from {}: {}",
            config.cert.public_key_path, err
        ),
    }
    server_manage
}

/// Serve a static directory from the file system.
/// This is intended to be used for OpenAPI frontends such as [Swagger](https://swagger.io/) or [RapiDoc](https://rapidocweb.com/).
/// The directory will be served iff [Config::serve_static_directory] is set.
/// If the directory does not exist on the filesystem while the configuration says it should be served, this function will panic.
/// When requesting the base of the [Config::static_directory_url], the `index.html` will be returned.
///
/// # Arguments
///
/// * `rocket`: the state of the application to configure
///
/// returns: Rocket<Build> the (configured) application state
fn configure_static_directory(rocket: Rocket<Build>) -> Rocket<Build> {
    let config: Config = rocket.figment().extract().expect("config");
    if config.serve_static_directory {
        info!(
            "Mount static directory '{}' to '{}'",
            config.static_directory_path, config.static_directory_url
        );
        rocket.mount(
            config.static_directory_url,
            FileServer::new(
                config.static_directory_path,
                Options::Index | Options::NormalizeDirs,
            ),
        )
    } else {
        rocket
    }
}

fn openapi_settings() -> OpenApiSettings {
    Default::default()
}

fn custom_openapi_spec(rocket: &Rocket<Build>) -> OpenApi {
    let rocket_config: rocket::Config = rocket.figment().extract().expect("config");
    let config: Config = rocket.figment().extract().expect("config");
    use okapi::openapi3::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "OpenKeg".to_owned(),
            description: Some("The backend API for the Musikverein Leopoldsdorf!".to_owned()),
            terms_of_service: Some(
                "https://github.com/mvl-at/keg/blob/master/license.adoc".to_owned(),
            ),
            contact: Some(Contact {
                name: Some("Richard Stöckl".to_owned()),
                url: Some("https://github.com/mvl-at/openkeg".to_owned()),
                email: Some("richard.stoeckl@aon.at".to_owned()),
                ..Default::default()
            }),
            license: Some(License {
                name: "GNU Free Documentation License 1.3".to_owned(),
                url: Some("https://www.gnu.org/licenses/fdl-1.3-standalone.html".to_owned()),
                ..Default::default()
            }),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        servers: vec![
            Server {
                url: config.openapi_url,
                description: Some("Self Hosted Instance".to_owned()),
                ..Default::default()
            },
            Server {
                url: format!("http://localhost:{}/api/v1/", rocket_config.port),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: "https://keg.mvl.at/api/v1/".to_owned(),
                description: Some("Sample Production Server".to_owned()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
