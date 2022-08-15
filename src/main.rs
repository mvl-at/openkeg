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

/// Entrypoint for the rocket application.
#[rocket::main]
async fn main() {
    env_logger::init();
    info!(
        "Welcome to OpenKeg {} - The backend of the Musikverein Leopoldsdorf!",
        env!("CARGO_PKG_VERSION")
    );
    let rocket = configure_rocket(initialize_build_state()).await;
    match rocket.launch().await {
        Ok(_) => info!("Shutdown OpenKeg!"),
        Err(err) => error!("Failed to start: {}", err.to_string()),
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

/// Create a new [Rocket<Build>] and merge the application configuration into it.
///
/// returns: Rocket<Build> the fresh build state
fn initialize_build_state() -> Rocket<Build> {
    let figment = config::read_config().merge((
        "ident",
        Ident::try_new(keg_user_agent()).expect("Valid ident for server response"),
    ));
    rocket::custom(figment).attach(AdHoc::config::<Config>())
}

/// Compose all the configuration functions to allow a single call to configure the rocket build state.
/// It is recommended to use this with a fresh [Rocket<Build>] instance.
///
/// # Arguments
///
/// * `rocket`: the build state which should be configured
///
/// returns: Rocket<Build>
async fn configure_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    let configured_rocket = manage_database_client(manage_member_state(manage_keys(attach_cors(
        manage_server_info(mount_static_directory(mount_controller_routes(rocket))),
    ))))
    .await;
    register_user_sync_task(&configured_rocket);
    configured_rocket
}

/// Fetch the routes and OpenApi documentation from the submodules and attach it to the rocket build.
///
/// # Arguments
///
/// * `rocket`: the build state to attach the routes to
///
/// returns: Rocket<Build>
fn mount_controller_routes(mut rocket: Rocket<Build>) -> Rocket<Build> {
    let openapi_settings = openapi_settings();
    let (info_route, info_spec) = get_info_routes_and_docs(&openapi_settings);
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
fn mount_static_directory(rocket: Rocket<Build>) -> Rocket<Build> {
    let config = rocket_configuration(&rocket);
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

/// Instantiate a [ServerInfo] and let rocket manage it.
///
/// # Arguments
///
/// * `rocket`: the current rocket build state
///
/// returns: Rocket<Build>
fn manage_server_info(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.manage(ServerInfo::new())
}

/// Attach the cors fairing to the rocket build state,
///
/// # Arguments
///
/// * `rocket`: the build state to attach the cors fairing to
///
/// returns: Rocket<Build>
fn attach_cors(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.attach(Cors)
}

/// Let the server manage the private and the public key.
/// Warnings will be printed to the log if this operation fails.
///
/// # Arguments
///
/// * `rocket`: the server where to register the keys
///
/// returns: Rocket<Build>
fn manage_keys(rocket: Rocket<Build>) -> Rocket<Build> {
    let config = rocket_configuration(&rocket);
    info!("Read the public and the private key");
    let mut server_manage = rocket;
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

/// Create an empty [MemberStateMutex] and let the rocket build state manage it.
/// This allows the application to provide the member state in the controller calls.
///
/// # Arguments
///
/// * `rocket`: the build state to attach the [MemberStateMutex] to
///
/// returns: Rocket<Build>
fn manage_member_state(rocket: Rocket<Build>) -> Rocket<Build> {
    let member_state = MemberState::mutex();
    rocket.manage(member_state)
}

/// Initialize the database client and let the rocket build state manage it.
///
/// # Arguments
///
/// * `rocket`: the build state to let manage the database client
///
/// returns: Rocket<Build>
async fn manage_database_client(rocket: Rocket<Build>) -> Rocket<Build> {
    let config = &rocket_configuration(&rocket);
    rocket.manage(initialize_client(config).await)
}

/// Create a new task which synchronizes the member state with the directory server in the interval given in the [Config].
/// If there is no [MemberStateMutex] managed by the rocket build state, a warning will be printed and nothing will happen.
/// This means that [manage_member_state] should be called with the build state first.
///
/// # Arguments
///
/// * `rocket`: the rocket build state to fetch the [MemberStateMutex] from
///
/// returns: ()
fn register_user_sync_task(rocket: &Rocket<Build>) {
    let config = rocket_configuration(rocket);
    let member_state_option = rocket.state::<MemberStateMutex>();
    if member_state_option.is_none() {
        warn!("Unable to retrieve member state, scheduled user synchronization will not work. Was 'manage_member_state()' called before?");
        return;
    }
    let mut member_state_clone = member_state_option
        .expect("Member state for synchronizing")
        .clone();
    task::spawn(async move {
        member_synchronization_task(&config, &mut member_state_clone).await;
    });
}

/// Retrieve the configuration from the current rocket build state.
/// If the configuration cannot be extracted, this function will panic.
///
/// # Arguments
///
/// * `rocket`: the build state to retrieve the configuration from
///
/// returns: Config
fn rocket_configuration(rocket: &Rocket<Build>) -> Config {
    rocket.figment().extract().expect("config")
}

/// Provide the OpenApi settings to be used in this application.
///
/// returns: OpenApiSettings
fn openapi_settings() -> OpenApiSettings {
    Default::default()
}

/// Create an [OpenApi] structure to use in this application.
/// This structure will contain the header such as the license, author and server list.
///
/// # Arguments
///
/// * `rocket`: the build state to retrieve the configuration from
///
/// returns: OpenApi
fn custom_openapi_spec(rocket: &Rocket<Build>) -> OpenApi {
    let rocket_config: rocket::Config = rocket.figment().extract().expect("config");
    let config = rocket_configuration(rocket);
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
