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

use rocket::State;
use rocket_okapi::openapi;

use crate::auth::authenticate;
use crate::user::auth::{AuthenticationResponder, BasicAuth};
use crate::user::key::PrivateKey;
use crate::user::tokens::generate_token;
use crate::{Config, MemberStateMutex};

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
/// * `private_key`: the private key to sign the jwt with
/// * `member_state`: the current member state
/// * `config`: the application configuration
///
/// returns: Result<Json<()>, Error>
#[openapi(tag = "Self Service")]
#[get("/login")]
pub async fn login(
    auth: BasicAuth,
    private_key: &State<PrivateKey>,
    member_state: &State<MemberStateMutex>,
    config: &State<Config>,
) -> AuthenticationResponder {
    let mut member_state_clone = member_state.inner().clone();
    authenticate(
        config,
        &mut member_state_clone,
        &auth.username,
        &auth.password,
    )
    .await
    .map_or_else(
        |err| {
            info!("Failed to authenticate: {}", err);
            AuthenticationResponder {
                request_token: None,
                renewal_token: None,
                request_token_required: true,
                renewal_token_required: true,
            }
        },
        |member| {
            debug!("Authenticated user: {}", member.username);
            let (request_token, renewal_token) = (
                generate_token(&member, true, config, private_key),
                generate_token(&member, false, config, private_key),
            );
            debug!(
                "Generated tokens {:?} and {:?}",
                request_token, renewal_token
            );
            AuthenticationResponder {
                request_token: request_token.ok(),
                renewal_token: renewal_token.ok(),
                request_token_required: true,
                renewal_token_required: true,
            }
        },
    )
}
