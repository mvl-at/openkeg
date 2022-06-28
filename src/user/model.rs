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

use okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authorization_option = request.headers().get_one("Authorization");
        if authorization_option.is_none() {
            debug!("skip credentials");
            return Outcome::Forward(());
        }
        let authorization = String::from(authorization_option.unwrap());
        if !authorization.starts_with("Basic ") {
            debug!("header does not start with basic");
            return Outcome::Forward(());
        }
        let result = base64::decode(authorization.replace("Basic ", ""));
        if result.is_err() {
            debug!("cannot base64 decode credentials");
            return Outcome::Failure((Status::BadRequest, ()));
        }
        let user_password_pair_result = String::from_utf8(result.unwrap());
        if user_password_pair_result.is_err() {
            debug!("credentials are not valid UTF-8");
            return Outcome::Failure((Status::BadRequest, ()));
        }
        let user_password_pair = user_password_pair_result.unwrap();
        let mut parts = user_password_pair.splitn(2, ":");
        let username = parts.next();
        let password = parts.next();
        if username.is_none() || password.is_none() {
            debug!("credentials do not contain a colon");
            return Outcome::Failure((Status::BadRequest, ()));
        }
        let basic_auth = BasicAuth {
            username: username.unwrap().to_string(),
            password: password.unwrap().to_string(),
        };
        Outcome::Success(basic_auth)
    }
}

impl<'r> OpenApiFromRequest<'r> for BasicAuth {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut security_req = SecurityRequirement::new();
        // Each security requirement needs to be met before access is allowed.
        security_req.insert("login".to_owned(), Vec::new());
        rocket_okapi::Result::Ok(RequestHeaderInput::Security(
            "login".to_string(),
            SecurityScheme {
                description: Some("Required for the login".to_string()),
                data: SecuritySchemeData::Http {
                    scheme: "Basic".to_string(),
                    bearer_format: Some("Basic".to_string()),
                },
                extensions: Object::default(),
            },
            security_req,
        ))
    }
}
