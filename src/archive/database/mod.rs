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

use reqwest::{Client, Method, RequestBuilder, StatusCode, Url};
use rocket::http::Status;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::Error;
use crate::Config;

#[derive(Deserialize)]
struct CouchError {
    error: String,
    reason: String,
}

impl CouchError {
    fn into_error(self, status: StatusCode) -> Error {
        Error {
            err: self.error,
            msg: Some(self.reason),
            http_status_code: status.as_u16(),
        }
    }
}

fn request_error<T>() -> Result<T, Error> {
    Err(Error {
        err: "Request Error".to_string(),
        msg: Some("The backend is unable to perform the request against the database".to_string()),
        http_status_code: Status::InternalServerError.code,
    })
}

/// Request a resource from the couch database.
/// If anything goes wrong during `URL`, request build or body deserialization, an appropriate [`Error`] will be returned which can be passed to the clients.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the client to use for the database request, likely is required to be authenticated with a cookie
/// * `request`: the [`RequestBuilder`] used to build the request, should already contain information such as the body
/// * `method`: the `HTTP` method used for the request - replaces the current one
/// * `api_url`: the `URL` relative to the base `URL` of the database
/// * `parameters`: the query parameters being used for the request
///
/// returns: Result<R, Error>
async fn request<R, P>(
    conf: &Config,
    client: &Client,
    request: RequestBuilder,
    method: Method,
    api_url: &String,
    parameters: &P,
) -> Result<R, Error>
where
    P: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let url = format!("{}{}", conf.database.url, api_url);
    let url_result = Url::parse(&*url);
    if url_result.is_err() {
        warn!(
            "Unable to parse URL '{}' provided by the application: {}",
            url,
            url_result.err().unwrap()
        );
        return request_error();
    }
    let request_result = request.query(parameters).build();
    if request_result.is_err() {
        warn!(
            "Unable to build the request provided by the application: {}",
            request_result.err().unwrap()
        );
        return request_error();
    }
    let mut request_success = request_result.unwrap();
    (*request_success.method_mut()) = method;
    let response_result = client.execute(request_success).await;
    if response_result.is_err() {
        warn!(
            "Unable to execute the request provided by the application: {}",
            response_result.err().unwrap()
        );
        return request_error();
    }
    let response = response_result.unwrap();
    let status = response.status();
    if !status.is_success() {
        let result = response.json::<CouchError>().await;
        if result.is_err() {
            warn!(
                "Unable to return error to client: {}",
                result.err().unwrap()
            );
            return request_error();
        }
        return Err(result.unwrap().into_error(status));
    }
    let deserialized_result = response.json::<R>().await;
    if deserialized_result.is_err() {
        warn!(
            "Unable to deserialize a response from the database: {}",
            deserialized_result.err().unwrap()
        );
        return request_error();
    }
    Ok(deserialized_result.unwrap())
}
