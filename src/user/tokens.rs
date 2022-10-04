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

use chrono::Duration;
use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::{Deserialize, Serialize};
use rocket::Request;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

use crate::member::model::Member;
use crate::member::state::{AllMembers, Repository};
use crate::user::auth::bearer_documentation;
use crate::user::key::{PrivateKey, PublicKey};
use crate::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: String,
    pub(crate) iss: String,
    pub(crate) exp: u64,
    pub(crate) ren: bool,
    _private: (),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        if auth_header.is_none() {
            debug!("Request does not contain Authorization header");
            return Forward(());
        }
        let bearer = String::from(auth_header.expect("Authentication header"));
        let token_optional = bearer.strip_prefix("Bearer ");
        if token_optional.is_none() {
            debug!("Token does not start with Bearer");
            return Forward(());
        }
        let token = token_optional.expect("Stripped token");
        let public_key = request.rocket().state::<PublicKey>();
        if let Some(pk) = public_key {
            let claims_result = decode_claims(token, pk);
            match claims_result {
                Ok(claims) => Success(claims),
                Err(err) => {
                    warn!(
                        "Provided a token which cannot be validated, maybe it is expired: {}",
                        err
                    );
                    Failure((Status::Unauthorized, ()))
                }
            }
        } else {
            warn!("Unable to retrieve public key, requests using authentication will not work");
            return Forward(());
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for Claims {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        bearer_documentation()
    }
}

/// Function to generate a jwt token.
/// This returns the [`Claims`] struct and the encoded value.
///
/// # Arguments
///
/// * `member`: the member which should be the subject of the token
/// * `renewal`: `true` if the token should be a refresh token or `false` if it should be a request token
/// * `config`: the application configuration
/// * `private_key`: the private key to sign the token with
///
/// returns: Result<(Claims, String), ()>
pub(crate) fn generate_token(
    member: &Member,
    renewal: bool,
    config: &Config,
    private_key: &PrivateKey,
) -> Result<(Claims, String), ()> {
    let duration = renewal
        .then(|| Duration::hours(config.jwt.renewal_expiration))
        .unwrap_or_else(|| Duration::minutes(config.jwt.expiration));
    let expiration = chrono::Local::now()
        .checked_add_signed(duration)
        .expect("valid timestamp");
    let claims = Claims {
        sub: member.username.to_string(),
        iss: config.jwt.issuer.to_string(),
        exp: expiration.timestamp() as u64,
        ren: renewal,
        _private: (),
    };
    debug!("Private key length: {}", &private_key.0.len());
    let encoding_key = &EncodingKey::from_rsa_pem(private_key.0.as_slice()).map_err(|e| {
        warn!(
            "Cannot decode private key, authentication will not work: {}",
            e
        )
    })?;
    jsonwebtoken::encode(&Header::new(Algorithm::RS512), &claims, encoding_key)
        .map(|encoded| (claims, encoded))
        .map_err(|e| warn!("Encoding error: {}", e))
}

/// Function to get the member from [`Claims`].
/// The member will be searched by their username.
/// If no user can be found, an error will be returned.
/// This function does not check for any token validity.
///  
/// # Arguments
///
/// * `claims`: the jwt to validate
/// * `renewal`: `true` of the token is expected to be a refresh token or `false` it is expected to be a request token
/// * `member`: the member of the application
///
/// returns: Result<Member, ()>
pub(crate) fn member_from_claims(
    claims: Claims,
    renewal: bool,
    members: &AllMembers,
) -> Result<Member, Error> {
    if claims.ren != renewal {
        info!("Tried to use refresh as request token or vice versa");
        return Err(Error::from(ErrorKind::InvalidToken));
    }
    info!(
        "Token issued by {} for {}, try to find member",
        claims.iss, claims.sub
    );
    members
        .find(&claims.sub)
        .cloned()
        .ok_or_else(|| Error::from(ErrorKind::InvalidToken))
}

/// Decode a [`Claims`] from a Bearer token string.
/// This function also validates the content of the token i.e. the expiration and the signature.
/// The token string must be in the raw JWT format, i.e. without a `Bearer ` prefix.
///
/// Validity in this context means:
///
///  * the token syntax is correct
///  * the token is not expired
///  * the token is signed by the key used by this application
///  * the token is expected to be a request/refresh token and is actual one
///
/// # Arguments
///
/// * `token`: the token string to decode
/// * `public_key`: the public key used for the signature verification
///
/// returns: Result<Claims, Error>
pub(crate) fn decode_claims(token: &str, public_key: &PublicKey) -> Result<Claims, Error> {
    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["iss", "sub", "ren", "exp"]);
    validation.algorithms = vec![Algorithm::RS512];
    debug!("Public key length: {}", &public_key.0.len());
    let decoding_key = &DecodingKey::from_rsa_pem(public_key.0.as_slice()).map_err(|e| {
        warn!(
            "Cannot decode public key, authentication will not work: {}",
            e
        );
        e
    })?;
    let claims = jsonwebtoken::decode::<Claims>(token, decoding_key, &validation)
        .map_err(|e| {
            info!("Cannot validate token: {}", e);
            e
        })?
        .claims;
    Ok(claims)
}
