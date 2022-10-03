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

use okapi::openapi3::OpenApi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;

/// The module for the authentication process.
/// Contains mostly implementations to deserialize structures from request during the authentication process.
pub mod auth;
/// Manage all the executives by their roles and manage the deserialization.
pub mod executives;
/// The module to manage the private and public keys used to sign and verify JWT signatures.
pub mod key;
/// A controller module for endpoints which provides self-service functionality to the user.
mod self_service;
/// Module to manage JWTs.
/// Contains the possibility to generate and verify them.
pub mod tokens;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: self_service::login,
        self_service::login_with_renewal,
        self_service::info,
        self_service::executive_roles
    ]
}
