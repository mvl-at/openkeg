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

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::result::Result as StdResult;

use okapi::openapi3::OpenApi;
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
    serde::json::Json,
    Build, Rocket,
};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::{openapi3::Responses, schemars},
    response::OpenApiResponderInner,
    OpenApiError,
};

use crate::Config;

/// A wrapper for the standard [StdResult] which contains a json body and an [ApiError].
pub type ApiResult<T> = StdResult<Json<T>, ApiError>;

/// Trait which purpose is to provide an example for the OpenApi specification.
pub trait SchemaExample {
    /// Provides an example instance for the type.
    ///
    /// returns: Self
    fn example() -> Self;
}

/// Create a map with a bunch of default HTTP status code descriptions.
///
/// returns: HashMap<&'static str, &'static str>
fn http_code_descriptions() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (
            "400",
            "[Bad Request](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400).
                The request given is wrongly formatted or data asked could not be fulfilled.",
        ),
        (
            "401",
            "[Not Authorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401).
                This response is given when a request requires authentication but none was issued.",
        ),
        (
            "404",
            "[Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404).
                This response is given when you request a page that does not exists.",
        ),
        (
            "422",
            "[Unprocessable Entity](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/422).
                This response is given when you request body is not correctly formatted.",
        ),
        (
            "500",
            "[Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500).
                This response is given when something wend wrong on the server.",
        ),
    ])
}

/// Error messages returned to user
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ApiError {
    /// The title of the error message
    pub err: String,
    /// The description of the error
    pub msg: Option<String>,
    /// HTTP Status Code returned
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for ApiError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiResponse};
        let resp = http_code_descriptions()
            .iter()
            .map(|(code, desc)| {
                (
                    code.to_string(),
                    RefOr::Object(OpenApiResponse {
                        description: desc.to_string(),
                        ..Default::default()
                    }),
                )
            })
            .collect();
        Ok(Responses {
            responses: resp,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Error `{}`: {}",
            self.err,
            self.msg.as_deref().unwrap_or("<no message>")
        )
    }
}

impl Error for ApiError {}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).expect("Error body");
        Response::build()
            .sized_body(body.len(), io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

/// Map an [`Error`] with a [`Status`] from the [`std::io`] to an [`ApiError`].
/// Since the error type of the module is private, only the [`io::Result`] can be converted.
/// The message and the error kind will be taken from the io error.
///
/// # Arguments
///
/// * `result`: the io result which contains the [`Error`] to map
/// * `status`: the status to use for the mapped [`ApiError`]
///
/// returns: Result<T, ApiError>
pub fn map_io_err<T>(result: io::Result<T>, status: Status) -> Result<T, ApiError> {
    result.map_err(|e| ApiError {
        err: e.to_string(),
        msg: Some(e.kind().to_string()),
        http_status_code: status.code,
    })
}

impl From<rocket::serde::json::Error<'_>> for ApiError {
    fn from(err: rocket::serde::json::Error) -> Self {
        use rocket::serde::json::Error::*;
        match err {
            Io(io_error) => ApiError {
                err: "IO Error".to_owned(),
                msg: Some(io_error.to_string()),
                http_status_code: 422,
            },
            Parse(_raw_data, parse_error) => ApiError {
                err: "Parse Error".to_owned(),
                msg: Some(parse_error.to_string()),
                http_status_code: 422,
            },
        }
    }
}

/// Provide the OpenApi settings to be used in this application.
///
/// returns: OpenApiSettings
pub fn openapi_settings() -> OpenApiSettings {
    Default::default()
}

/// Create an [OpenApi] structure to use in this application.
/// This structure will contain the header such as the license, author and server list.
///
/// # Arguments
///
/// * `rocket`: the build state to retrieve the configuration from
///
/// returns: OpenApi
pub fn custom_openapi_spec(rocket: &Rocket<Build>) -> OpenApi {
    let rocket_config: rocket::Config = rocket.figment().extract().expect("rocket config");
    let config: Config = rocket.figment().extract().expect("config");
    use okapi::openapi3::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "OpenKeg".to_owned(),
            description: Some("The backend API for the Musikverein Leopoldsdorf!".to_owned()),
            terms_of_service: Some(
                "https://github.com/mvl-at/keg/blob/master/license.adoc".to_owned(),
            ),
            contact: Some(Contact {
                name: Some("Richard Stöckl".to_owned()),
                url: Some("https://github.com/mvl-at/openkeg".to_owned()),
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
                url: config.openapi_url,
                description: Some("Self Hosted Instance".to_owned()),
                ..Default::default()
            },
            Server {
                url: format!("http://localhost:{}/api/v1/", rocket_config.port),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: "https://keg.mvl.at/api/v1/".to_owned(),
                description: Some("Sample Production Server".to_owned()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
