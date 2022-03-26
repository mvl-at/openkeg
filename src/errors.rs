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

use std::collections::HashMap;

use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
};
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::okapi::schemars::{self};
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner, OpenApiError};

pub type Result<T> = std::result::Result<rocket::serde::json::Json<T>, Error>;

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
pub struct Error {
    /// The title of the error message
    pub err: String,
    /// The description of the error
    pub msg: Option<String>,
    /// HTTP Status Code returned
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for Error {
    fn responses(
        _generator: &mut OpenApiGenerator,
    ) -> std::result::Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};
        let resp = http_code_descriptions()
            .iter()
            .map(|(code, desc)| {
                (
                    code.to_string(),
                    RefOr::Object(OpenApiReponse {
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

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Error `{}`: {}",
            self.err,
            self.msg.as_deref().unwrap_or("<no message>")
        )
    }
}

impl std::error::Error for Error {}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

impl From<rocket::serde::json::Error<'_>> for Error {
    fn from(err: rocket::serde::json::Error) -> Self {
        use rocket::serde::json::Error::*;
        match err {
            Io(io_error) => Error {
                err: "IO Error".to_owned(),
                msg: Some(io_error.to_string()),
                http_status_code: 422,
            },
            Parse(_raw_data, parse_error) => Error {
                err: "Parse Error".to_owned(),
                msg: Some(parse_error.to_string()),
                http_status_code: 422,
            },
        }
    }
}
