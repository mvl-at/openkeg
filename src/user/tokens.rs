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
use rocket::serde::{Deserialize, Serialize};

use crate::member::model::Member;
use crate::member::state::{AllMembers, Repository};
use crate::user::key::{PrivateKey, PublicKey};
use crate::Config;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    exp: u64,
    ren: bool,
}

/// Function to generate a jwt token.
///
/// # Arguments
///
/// * `member`: the member which should be the subject of the token
/// * `renewal`: `true` if the token should be a refresh token or `false` if it should be a request token
/// * `config`: the application configuration
/// * `private_key`: the private key to sign the token with
///
/// returns: Result<String, ()>
pub fn generate_token(
    member: &Member,
    renewal: bool,
    config: &Config,
    private_key: &PrivateKey,
) -> Result<String, ()> {
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
    };
    debug!("Private key length: {}", &private_key.0.len());
    jsonwebtoken::encode(
        &Header::new(Algorithm::RS512),
        &claims,
        &EncodingKey::from_rsa_der(&private_key.0),
    )
    .map_err(|e| warn!("Encoding error: {}", e))
}

/// Function to validate a jwt token.
/// If the token is valid, the corresponding `Member` will be returned.
/// Validity in this context means:
///
///  * the token syntax is correct
///  * the token is not expired
///  * the token is signed by the key used by this application
///  * the token is expected to be a request/refresh token and is actual one
///  
/// # Arguments
///
/// * `token`: the jwt to validate
/// * `renewal`: `true` of the token is expected to be a refresh token or `false` it is expected to be a request token
/// * `member`: the member of the application
/// * `public_key`: the public key used to ensure the signature
///
/// returns: Result<Member, ()>
pub fn validate_token(
    token: &str,
    renewal: bool,
    members: &AllMembers,
    public_key: &PublicKey,
) -> Result<Member, Error> {
    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["iss", "sub", "ren", "exp"]);
    validation.algorithms = vec![Algorithm::RS512];
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_rsa_der(&public_key.0),
        &validation,
    )
    .map_err(|e| {
        info!("Cannot validate token: {}", e);
        e
    })?
    .claims;
    if claims.ren != renewal {
        info!("Tried to use refresh as request token or vice versa");
        return Err(Error::from(ErrorKind::InvalidToken));
    }
    info!(
        "Token issued by {} for {} is valid, try to find member",
        claims.iss, claims.sub
    );
    members
        .find(&claims.sub)
        .cloned()
        .ok_or_else(|| Error::from(ErrorKind::InvalidToken))
}
