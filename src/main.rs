// Keg, the lightweight backend of the Musikverein Leopoldsdorf.
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

use figment::Figment;
use log::{error, info};
use okapi::openapi3::OpenApi;
use rocket::fairing::AdHoc;
use rocket::{Build, Rocket};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{mount_endpoints_and_merged_docs, swagger_ui::*};

use crate::members::ldap;

mod archive;
mod config;
mod errors;
mod members;
mod schema_util;

#[rocket::main]
async fn main() {
    env_logger::init();
    let figment = config::read_config();
    let server_result = create_server(figment);
    match server_result.launch().await {
        Ok(_) => info!("shutdown keg!"),
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
        .attach(AdHoc::config::<config::Config>());
    mount_endpoints_and_merged_docs! {
        rocket, "/api/v1".to_owned(), openapi_settings,
        "/" => custom_route_spec,
        "/archive" => archive::get_routes_and_docs(&openapi_settings),
        "/members" => members::get_routes_and_docs(&openapi_settings)
    };
    rocket
}

fn openapi_settings() -> OpenApiSettings {
    rocket_okapi::settings::OpenApiSettings::default()
}

fn custom_openapi_spec() -> OpenApi {
    use okapi::openapi3::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "Keg".to_owned(),
            description: Some("The backend API for the Musikverein Leopoldsdorf!".to_owned()),
            terms_of_service: Some(
                "https://github.com/mvl-at/keg/blob/master/license.adoc".to_owned(),
            ),
            contact: Some(Contact {
                name: Some("Richard Stöckl".to_owned()),
                url: Some("https://github.com/mvl-at/keg".to_owned()),
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
