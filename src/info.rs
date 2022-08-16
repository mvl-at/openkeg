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

use crate::keg_user_agent;
use crate::openapi::{ApiResult, SchemaExample};
use chrono::Local;
use okapi::openapi3::OpenApi;
use okapi::schemars::JsonSchema;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use serde::{Deserialize, Serialize};

/// A structure to provide basic information about the server.
/// This is intended to determine if the server is up or not.
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct ServerInfo {
    /// The RFC3339 compliant date time when this instance was started at.
    start: String,
    /// The version of this server software.
    version: String,
    /// Whether this server is run in debug mode or not.
    debug: bool,
}

impl ServerInfo {
    /// Create a new instance of the server information.
    /// The start will be set to the time when this function is called.
    pub fn new() -> Self {
        Self {
            version: keg_user_agent(),
            start: Local::now().to_rfc3339(),
            debug: cfg!(feature = "debug"),
        }
    }
}

impl SchemaExample for ServerInfo {
    fn example() -> Self {
        Self {
            start: Local::now().to_rfc3339(),
            version: keg_user_agent(),
            debug: false,
        }
    }
}

/// Return the current information of the server using its internal state.
///
/// # Arguments
///
/// * `info_state`: the state of the server
///
/// returns: Result<Json<ServerInfo>, Error>
#[openapi(tag = "Misc")]
#[get("/")]
pub fn info(info_state: &State<ServerInfo>) -> ApiResult<ServerInfo> {
    Ok(Json((*info_state).clone()))
}

/// Generate the OpenApi documentation and routes for the info endpoint.
///
/// # Arguments
///
/// * `settings`: the OpenApi settings to use to generate the documentation
///
/// returns: (Vec<Route, Global>, OpenApi)
pub fn get_info_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: info,]
}
