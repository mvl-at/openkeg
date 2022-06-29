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

use std::sync::Arc;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::sync::RwLock;
use rocket::State;
use rocket_okapi::openapi;

use crate::errors::{Error, Result};
use crate::ldap::authenticate;
use crate::user::model::BasicAuth;
use crate::{Config, MemberState, MemberStateMutex};

/// Login the user.
/// On success, this generates two keys:
///
/// * request token: a jwt for usage for requests which require authentication
/// * refresh token: a jwt which can only be used to generate a new request tokens
///
/// The request token expires much earlier than the refresh token which means that applications should only store the refresh token permanently and then gather a new request token when required.
/// Instead of returning them via the body, the response will attach the request token into the `Authorization` and the refresh token into the `Authorization-Renewal` headers.
/// Note that both header values will be prefixed with `Bearer `.
/// Despite being required for future requests, this prefix needs to be removed before deserialization.  
///
/// # Arguments
///
/// * `auth`: the structure which holds the credentials to use for authentication
/// * `config`: the application configuration
/// * `member_state`: the current member state
///
/// returns: Result<Json<()>, Error>
#[openapi(tag = "Self Service")]
#[get("/login")]
pub async fn login(
    auth: BasicAuth,
    config: &State<Config>,
    member_state: &State<MemberStateMutex>,
) -> Result<()> {
    let mut member_state_clone = member_state.inner().clone();
    let member_result = authenticate(
        config,
        &mut member_state_clone,
        &auth.username,
        &auth.password,
    )
    .await;
    if member_result.is_ok() {
        debug!("authenticated user: {:?}", member_result.unwrap());
        Ok(Json(()))
    } else {
        Err(Error {
            err: "authentication error".to_string(),
            msg: Some("Unable to authenticate the user".to_string()),
            http_status_code: Status::Forbidden.code,
        })
    }
}
