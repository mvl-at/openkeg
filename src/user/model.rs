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

use crate::errors::Error;
use okapi::map;
use okapi::openapi3::{
    Object, ParameterValue, RefOr, Response, Responses, SecurityRequirement, SecurityScheme,
    SecuritySchemeData,
};
use rocket::http::{ContentType, Header, Status};
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::Request;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::response::OpenApiResponderInner;
use std::io::Cursor;

use crate::members::model::Member;
use crate::user::key::PublicKey;
use crate::user::tokens::validate_token;
use crate::MemberStateMutex;

#[non_exhaustive]
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
            return Forward(());
        }
        let authorization = String::from(authorization_option.unwrap());
        if !authorization.starts_with("Basic ") {
            debug!("header does not start with basic");
            return Forward(());
        }
        let result = base64::decode(authorization.replace("Basic ", ""));
        if result.is_err() {
            debug!("cannot base64 decode credentials");
            return Failure((Status::BadRequest, ()));
        }
        let user_password_pair_result = String::from_utf8(result.unwrap());
        if user_password_pair_result.is_err() {
            debug!("credentials are not valid UTF-8");
            return Failure((Status::BadRequest, ()));
        }
        let user_password_pair = user_password_pair_result.unwrap();
        let mut parts = user_password_pair.splitn(2, ":");
        let username = parts.next();
        let password = parts.next();
        if username.is_none() || password.is_none() {
            debug!("credentials do not contain a colon");
            return Failure((Status::BadRequest, ()));
        }
        let basic_auth = BasicAuth {
            username: username.unwrap().to_string(),
            password: password.unwrap().to_string(),
        };
        Success(basic_auth)
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
        Ok(RequestHeaderInput::Security(
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Member {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        if auth_header.is_none() {
            debug!("request does not contain Authorization header");
            return Forward(());
        }
        let bearer = String::from(auth_header.unwrap());
        if !bearer.starts_with("Bearer ") {
            debug!("token does not start with Bearer");
            return Forward(());
        }
        let token = bearer.replace("Bearer ", "");
        let members = request.rocket().state::<MemberStateMutex>();
        if members.is_none() {
            warn!("unable to retrieve members, requests using authentication will not work");
            return Forward(());
        }
        let public_key = request.rocket().state::<PublicKey>();
        if public_key.is_none() {
            warn!("unable to retrieve public key, requests using authentication will not work");
            return Forward(());
        }
        let all_members = members.unwrap().read().await;
        let member = validate_token(&token, false, &all_members.all_members, public_key.unwrap());
        if member.is_err() {
            debug!("token was invalid");
            return Failure((Status::Unauthorized, ()));
        }
        Success(member.unwrap())
    }
}

/// A responder for the authentication header and corresponding error.
pub struct AuthenticationResponder {
    pub(crate) request_token: Option<String>,
    pub(crate) renewal_token: Option<String>,
    pub(crate) request_token_required: bool,
    pub(crate) renewal_token_required: bool,
}

fn authorization_error() -> Error {
    Error {
        err: "Authentication Failure".to_string(),
        msg: Some("Something went wrong during the authentication either wrong credentials or server errors, due to security reasons no more details are provided.".to_string()),
        http_status_code: Status::Unauthorized.code,
    }
}

impl<'r> Responder<'r, 'static> for AuthenticationResponder {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        if (self.request_token.is_none() && self.request_token_required)
            || (self.renewal_token.is_none() && self.renewal_token_required)
        {
            let body = serde_json::to_string(&authorization_error()).expect("serialized error");
            return rocket::response::Response::build()
                .sized_body(body.len(), Cursor::new(body))
                .header(ContentType::JSON)
                .status(Status::Unauthorized)
                .ok();
        }
        let mut response_builder = rocket::response::Response::build();
        response_builder.header(ContentType::Text);
        if self.request_token.is_some() {
            response_builder.header(Header::new(
                "Authorization",
                format!("Bearer {}", self.request_token.unwrap()),
            ));
        }
        if self.renewal_token.is_some() {
            response_builder.header(Header::new(
                "Authorization-Renewal",
                format!("Bearer {}", self.renewal_token.unwrap()),
            ));
        }
        response_builder.ok()
    }
}

impl OpenApiResponderInner for AuthenticationResponder {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use okapi::openapi3::Header;
        let auth_headers = map! {"Authorization".to_string() => RefOr::Object(Header{
            description: Some("the request token, prefixed with 'Bearer '".to_string()),
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Content {content: map!{}},
            extensions: map! {}
        }),"Authorization-Renewal".to_string() => RefOr::Object(Header{
            description: Some("the renewal token, prefixed with 'Bearer '".to_string()),
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Content {content: map!{}},
            extensions: map! {}
        })};
        let err_response = Response {
            description: "the authentication failed".to_string(),
            headers: map! {},
            content: map! {},
            links: map! {},
            extensions: map! {},
        };
        let auth_response = Response {
            description: "the authentication succeeded".to_string(),
            headers: auth_headers,
            content: map! {},
            links: map! {},
            extensions: map! {},
        };
        let responses = Responses {
            default: Some(RefOr::Object(auth_response.clone())),
            responses: map! {"200".to_string()=>RefOr::Object(auth_response), "401".to_string() => RefOr::Object(err_response)},
            extensions: Object::default(),
        };
        Ok(responses)
    }
}
