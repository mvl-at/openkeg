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

use reqwest::{Client, Method, RequestBuilder, StatusCode, Url};
use rocket::http::Status;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::api_result::Error;
use crate::database::authenticate;
use crate::schema_util::SchemaExample;
use crate::Config;

pub mod score;
pub mod statistic;

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

/// The response returned when performing a search.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct FindResponse<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The documents which match the requested filter.
    pub docs: Vec<D>,
    /// The bookmark used for pagination.
    pub bookmark: String,
    /// The execution statistics generated by the database.
    pub execution_stats: ExecutionStats,
}

impl<D> SchemaExample for FindResponse<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            docs: vec![],
            bookmark: "g1AAAABueJzLYWBgYMpgSmHgKy5JLCrJTq2MT8lPzkzJBYprFyfnF6UWW6WZWFgamhiZ6yYZG1jqmpglJ-smGhgZ6JokJ6WlWqYmp6ZZpoKM4IAZkQPUzAgygTcksyg_J7VIwTEFSGZlAQCcwx9S".to_string(),
            execution_stats: SchemaExample::example(),
        }
    }
}

/// Statistics which are generated by the database server.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct ExecutionStats {
    /// Number of index keys examined. Currently always 0.
    pub total_keys_examined: u64,
    /// Number of documents fetched from the database / index, equivalent to using include_docs=true in a view. These may then be filtered in-memory to further narrow down the result set based on the selector.
    pub total_docs_examined: u64,
    /// Number of documents fetched from the database using an out-of-band document fetch. This is only non-zero when read quorum > 1 is specified in the query parameters.
    pub total_quorum_docs_examined: u64,
    /// Number of results returned from the query. Ideally this should not be significantly lower than the total documents / keys examined.
    pub results_returned: u64,
    /// Total execution time in milliseconds as measured by the database.
    pub execution_time_ms: f64,
}

impl SchemaExample for ExecutionStats {
    fn example() -> Self {
        Self {
            total_keys_examined: 0,
            total_docs_examined: 0,
            total_quorum_docs_examined: 0,
            results_returned: 0,
            execution_time_ms: 0.0,
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

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct OperationResponse {
    /// The id of the deleted document.
    pub id: String,
    /// The status of the operation.
    pub ok: bool,
    /// The revision of the document of the operation context.
    pub rev: String,
}

impl SchemaExample for OperationResponse {
    fn example() -> Self {
        Self {
            id: "scores:s8eu".to_string(),
            ok: true,
            rev: "1-h98rgu".to_string(),
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
    let request_url = url_result.unwrap();
    debug!("The request URL is: {}", request_url);
    let request = client.request(method, request_url).query(parameters);
    let request_result = request_hook(request).build();
    if request_result.is_err() {
        warn!(
            "Unable to build the request provided by the application: {}",
            request_result.err().unwrap()
        );
        return request_error();
    }
    let request_success = request_result.unwrap();
    let request_clone_optional = request_success.try_clone();
    let response_result = client.execute(request_success).await;
    if response_result.is_err() {
        warn!(
            "Unable to execute the request provided by the application: {}",
            response_result.err().unwrap()
        );
        return request_error();
    }
    let mut response = response_result.unwrap();
    let mut status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        info!("The session cookie seems to be expired, try to reauthenticate");
        let auth_result = authenticate(conf, client).await;
        if auth_result.is_err() {
            return Err(Error {
                err: "Database Error".to_string(),
                msg: Some(
                    "Cannot connect to the database, please contact the administrator".to_string(),
                ),
                http_status_code: Status::InternalServerError.code,
            });
        }
        if request_clone_optional.is_none() {
            return Err(Error {
                err: "Database Error".to_string(),
                msg: Some(
                    "Unable to reproduce the request, you may try again immediately".to_string(),
                ),
                http_status_code: Status::ServiceUnavailable.code,
            });
        }
        let response_clone_result = client.execute(request_clone_optional.unwrap()).await;
        if response_clone_result.is_err() {
            warn!(
                "Unable to execute the second request provided by the application: {}",
                response_clone_result.err().unwrap()
            );
            return request_error();
        }
        response = response_clone_result.unwrap();
        status = response.status();
    }
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

/// Checks whether the provided [id] is part of the [partition] ot not.
///
/// # Arguments
///
/// * `id`: the id to check
/// * `partition`: the partition which could contain the [id]
///
/// returns: Option<Error>
fn check_score_partition(id: &String, partition: &String) -> Option<Error> {
    if id.starts_with(format!("{}:", partition).as_str()) {
        None
    } else {
        Some(Error {
            err: "invalid id".to_string(),
            msg: Some("the provided id starts with an invalid partition".to_string()),
            http_status_code: Status::UnprocessableEntity.code,
        })
    }
}