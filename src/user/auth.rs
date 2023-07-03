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

use std::io::Cursor;

use base64::{engine, Engine};
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

use crate::member::model::Member;
use crate::openapi::ApiError;
use crate::user::tokens::{
    member_from_claims, Claims, AUTHORIZATION_HEADER, AUTHORIZATION_RENEWAL_HEADER,
};
use crate::MemberStateMutex;

/// The basic auth structure as used in the HTTP protocol.
#[non_exhaustive]
pub struct BasicAuth {
    /// The username part of the header.
    pub username: String,
    /// The password part of the header.
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authorization_option = request.headers().get_one(AUTHORIZATION_HEADER);
        if authorization_option.is_none() {
            debug!("Skip credentials");
            return Forward(());
        }
        let authorization = String::from(authorization_option.expect("Authorization header"));
        if !authorization.starts_with("Basic ") {
            debug!("Header does not start with basic");
            return Forward(());
        }
        let result = engine::general_purpose::STANDARD.decode(authorization.replace("Basic ", ""));
        if let Err(err) = result {
            warn!("Cannot base64 decode credentials {}", err);
            return Failure((Status::BadRequest, ()));
        }
        let user_password_bytes = result.expect("Base64 decoded password");
        let user_password_pair = String::from_utf8_lossy(user_password_bytes.as_slice());
        let mut parts = user_password_pair.splitn(2, ':');
        let username = parts.next();
        let password = parts.next();
        if username.is_none() || password.is_none() {
            debug!("Credentials do not contain a colon");
            return Failure((Status::BadRequest, ()));
        }
        let basic_auth = BasicAuth {
            username: username.expect("Decoded username").to_string(),
            password: password.expect("Decoded password").to_string(),
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
        let members = request.rocket().state::<MemberStateMutex>();
        if members.is_none() {
            warn!("Unable to retrieve member, requests using authentication will not work");
            return Forward(());
        }
        let all_members = members.expect("Member read lock").read().await;
        let claims_outcome = Claims::from_request(request).await;
        match claims_outcome {
            Failure(fail) => Failure(fail),
            Forward(forward) => Forward(forward),
            Success(claims) => {
                let member = member_from_claims(claims, false, &all_members.all_members);
                if member.is_err() {
                    debug!("Token was invalid");
                    return Failure((Status::Unauthorized, ()));
                }
                Success(member.expect("Extracted Member from token"))
            }
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for Member {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        bearer_documentation()
    }
}

/// Generate the OpenAPI documentation for the bearer token requirement.
/// This function should be used in order to expose only a single field to the documentation to set the bearer token.
pub fn bearer_documentation() -> rocket_okapi::Result<RequestHeaderInput> {
    let mut security_req = SecurityRequirement::new();
    // Each security requirement needs to be met before access is allowed.
    security_req.insert("bearer token".to_owned(), Vec::new());
    Ok(RequestHeaderInput::Security(
        "bearer token".to_string(),
        SecurityScheme {
            description: Some("Required for requests which need authorization by a bearer token. Log in first to retrieve it".to_string()),
            data: SecuritySchemeData::Http {
                scheme: "bearer".to_string(),
                bearer_format: Some("JWT".to_string()),
            },
            extensions: Object::default(),
        },
        security_req,
    ))
}

/// A responder for the authentication header and corresponding error.
pub struct AuthenticationResponder {
    pub(crate) request_token: Option<String>,
    pub(crate) request_token_required: bool,
    pub(crate) renewal_token: Option<String>,
    pub(crate) renewal_token_required: bool,
}

/// A generic authentication error used to hide the real issue from the user.
/// The purpose is to make an attack more difficult than with a more verbose error.
pub(crate) fn authorization_error() -> ApiError {
    ApiError {
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
        if let Some(token) = self.request_token {
            response_builder.header(Header::new(
                AUTHORIZATION_HEADER,
                format!("Bearer {}", token),
            ));
        }
        if let Some(token) = self.renewal_token {
            response_builder.header(Header::new(
                AUTHORIZATION_RENEWAL_HEADER,
                format!("Bearer {}", token),
            ));
        }
        response_builder.ok()
    }
}

impl OpenApiResponderInner for AuthenticationResponder {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use okapi::openapi3::Header;
        let auth_headers = map! {AUTHORIZATION_HEADER.to_string() => RefOr::Object(Header{
            description: Some("The request token, prefixed with 'Bearer '".to_string()),
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Content {content: map!{}},
            extensions: map! {}
        }),AUTHORIZATION_RENEWAL_HEADER.to_string() => RefOr::Object(Header{
            description: Some("The renewal token, prefixed with 'Bearer '".to_string()),
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Content {content: map!{}},
            extensions: map! {}
        })};
        let err_response = Response {
            description: "The authentication failed".to_string(),
            headers: map! {},
            content: map! {},
            links: map! {},
            extensions: map! {},
        };
        let auth_response = Response {
            description: "The authentication succeeded".to_string(),
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
