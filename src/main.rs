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

use std::sync::Arc;

use figment::Figment;
use ldap3::tokio::task;
use okapi::openapi3::OpenApi;
use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::RwLock;
use rocket::{Build, Rocket};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{mount_endpoints_and_merged_docs, swagger_ui::*};

use crate::config::Config;
use crate::cors::CORS;
use crate::database::initialize_client;
use crate::ldap::auth;
use crate::ldap::sync::member_synchronization_task;
use crate::members::state::MemberState;
use crate::user::key::{read_private_key, read_public_key};

mod api_result;
mod archive;
mod config;
mod cors;
mod database;
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
    let figment = config::read_config();
    let member_state = MemberState::mutex();
    let mut server_result = create_server(figment).manage(member_state);
    server_result = manage_keys(server_result).attach(CORS);
    let config = server_result.figment().extract::<Config>().expect("config");
    server_result = server_result.manage(initialize_client(&config).await);
    register_user_sync_task(&server_result);
    match server_result.launch().await {
        Ok(_) => info!("Shutdown OpenKeg!"),
        Err(err) => error!("failed to start: {}", err.to_string()),
    }
}

fn create_server(figment: Figment) -> Rocket<Build> {
    let custom_route_spec = (vec![], custom_openapi_spec());
    let openapi_settings = openapi_settings();
    let mut rocket = rocket::custom(figment)
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .attach(AdHoc::config::<Config>());
    mount_endpoints_and_merged_docs! {
        rocket, "/api/v1".to_owned(), openapi_settings,
        "/" => custom_route_spec,
        "/scores" => archive::get_scores_routes_and_docs(&openapi_settings),
        "/books" => archive::get_books_routes_and_docs(&openapi_settings),
        "/statistics" => archive::get_statistics_routes_and_docs(&openapi_settings),
        "/members" => members::get_routes_and_docs(&openapi_settings),
        "/user" => user::get_routes_and_docs(&openapi_settings),
    };
    rocket
}

fn register_user_sync_task(server: &Rocket<Build>) {
    let config: Config = server.figment().extract().expect("config");
    let member_state_option = server.state::<MemberStateMutex>();
    if member_state_option.is_none() {
        warn!("Unable to retrieve member state, scheduled user synchronization will not work");
        return;
    }
    let mut member_state_clone = member_state_option.unwrap().clone();
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
    let private_key = read_private_key(&config);
    if private_key.is_err() {
        let err = private_key.err().unwrap();
        warn!(
            "Unable to read the private key from {}: {}",
            config.cert.private_key_path, err
        );
    } else {
        server_manage = server_manage.manage(private_key.unwrap());
        info!("Private key successfully added to application state")
    }
    let public_key = read_public_key(&config);
    if public_key.is_err() {
        let err = public_key.err().unwrap();
        warn!(
            "Unable to read the public key from {}: {}",
            config.cert.public_key_path, err
        );
    } else {
        server_manage = server_manage.manage(public_key.unwrap());
        info!("Public key successfully added to application state")
    }
    server_manage
}

fn openapi_settings() -> OpenApiSettings {
    rocket_okapi::settings::OpenApiSettings::default()
}

fn custom_openapi_spec() -> OpenApi {
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
                url: "http://localhost:8000/api/v1/".to_owned(),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: "https://keg.mvl.at/api/v1/".to_owned(),
                description: Some("Production Server".to_owned()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
