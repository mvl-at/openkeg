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
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::Error;
use crate::schema_util::SchemaExample;
use crate::Config;

pub mod score;

/// A page for pagination which is used for huge collections as the score archive.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct Pagination<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The size of the results vector.
    pub total_rows: u64,
    /// The offset where to begin to query.
    /// Starts with 0.
    pub offset: u64,
    /// The actual results.
    /// Will be empty when `offset >= total_rows`.
    pub rows: Vec<PaginationRow<D>>,
}

impl<D> SchemaExample for Pagination<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            total_rows: 150,
            offset: 150,
            rows: vec![],
        }
    }
}

/// A page for pagination which is used for huge collections as the score archive.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct PaginationRow<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The emitted id of the row
    pub id: String,
    /// The emitted key of the row
    pub key: String,
    /// The actual document of this row
    pub doc: D,
}

impl<D> SchemaExample for PaginationRow<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            id: "score:289j9f84".to_string(),
            key: "score:289j9f84".to_string(),
            doc: SchemaExample::example(),
        }
    }
}

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
/// * `request_hook`: a function used to modify the request, can be used to insert information such as the body
/// * `method`: the `HTTP` method used for the request - replaces the current one
/// * `api_url`: the `URL` relative to the base `URL` of the database
/// * `parameters`: the query parameters being used for the request
///
/// returns: Result<R, Error>
async fn request<'a, R, P>(
    conf: &Config,
    client: &Client,
    request_hook: Box<dyn FnOnce(RequestBuilder) -> RequestBuilder + Send + 'a>,
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
    let request = client
        .request(method, url_result.unwrap())
        .query(parameters);
    let request_result = request_hook(request).build();
    if request_result.is_err() {
        warn!(
            "Unable to build the request provided by the application: {}",
            request_result.err().unwrap()
        );
        return request_error();
    }
    let request_success = request_result.unwrap();
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
