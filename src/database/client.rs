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

use crate::openapi::{ApiError, SchemaExample};
use crate::{keg_user_agent, Config};
use reqwest::{Client, ClientBuilder, Method, RequestBuilder, StatusCode, Url};
use rocket::http::Status;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

/// An alias for the database HTTP client.
/// Used to be able to let rocket manage multiple HTTP clients, each for its specialized purpose.
/// May be replaced with a tuple struct in the future.
pub type DatabaseClient = Client;

/// Initialize the database client and configures it.
/// If the initialization fails this function will panic.
/// After the initialization this functions tries to authenticate against the database interface using cookies.
/// When this fails, an error will be printed.
///
/// # Arguments
///
/// * `conf`: the application configuration
///
/// returns: the configured [`DatabaseClient`]
pub async fn initialize_client(conf: &Config) -> DatabaseClient {
    let client = ClientBuilder::new()
        .user_agent(keg_user_agent().as_str())
        .cookie_store(true)
        .build()
        .map_err(|e| {
            error!("Unable to initialize http client: {}", e);
            e
        })
        .expect("First database client");
    authenticate(conf, &client)
        .await
        .map_err(|e| {
            error!("Unable to authenticate http client: {}", e);
            e
        })
        .expect("First authenticated client");
    client
}

/// Internal holder for username, password credentials.
/// Only used for convenience.
#[derive(Serialize)]
struct Credentials {
    /// The username.
    name: String,
    /// The password.
    password: String,
}

impl From<&Config> for Credentials {
    fn from(config: &Config) -> Self {
        Credentials {
            name: config.database.username.to_string(),
            password: config.database.password.to_string(),
        }
    }
}

/// The authentication function to perform an HTTP authentication request against the database server.
/// If the process was successful, the authentication cookie will be stored in the cookie store.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the HTTP client to use, cookie support is required
///
/// returns: ()
pub(crate) async fn authenticate(conf: &Config, client: &Client) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(&*format!(
        "{}{}",
        conf.database.url, conf.database.database_mapping.authentication
    ))?;
    let request = client.post(url).form(&<Credentials>::from(conf)).build()?;
    let response = client.execute(request).await?;
    response.error_for_status()?;
    info!("Authentication to the database interface was successful");
    Ok(())
}

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

/// The structure which CouchDB returns when an error happens.
/// For internal use only, should be mapped to an application wide error such as [ApiError] as soon as possible.
#[derive(Deserialize)]
struct DatabaseError {
    /// The error message.
    error: String,
    /// The reason why the error was triggered.
    reason: String,
}

impl From<(DatabaseError, StatusCode)> for ApiError {
    fn from((error, status): (DatabaseError, StatusCode)) -> Self {
        Self {
            err: error.error,
            msg: Some(error.reason),
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

/// Provide a generic error message when something went wrong during the database request.
/// This should only be used when no further error can be found out or should be hidden to the Rest interface consumer.
fn request_error() -> ApiError {
    ApiError {
        err: "Request Error".to_string(),
        msg: Some("The backend is unable to perform the request against the database".to_string()),
        http_status_code: Status::InternalServerError.code,
    }
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
pub(crate) async fn request<'a, R, P>(
    conf: &Config,
    client: &Client,
    request_hook: Box<dyn FnOnce(RequestBuilder) -> RequestBuilder + Send + 'a>,
    method: Method,
    api_url: &str,
    parameters: &P,
) -> Result<R, ApiError>
where
    P: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let url_string = format!("{}{}", conf.database.url, api_url);
    let url = Url::parse(&*url_string).map_err(|e| {
        warn!(
            "Unable to parse URL '{}' provided by the application: {}",
            url_string, e
        );
        request_error()
    })?;
    debug!("The request URL is: {}", url);
    let request_builder = client.request(method, url).query(parameters);
    let request = request_hook(request_builder).build().map_err(|e| {
        warn!(
            "Unable to build the request provided by the application: {}",
            e
        );
        request_error()
    })?;
    let request_clone_optional = request.try_clone();
    let mut response = client.execute(request).await.map_err(|e| {
        warn!(
            "Unable to execute the request provided by the application: {}",
            e
        );
        request_error()
    })?;
    let mut status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        info!("The session cookie seems to be expired, try to reauthenticate");
        authenticate(conf, client).await.map_err(|e| {
            warn!("Unable to re-authenticate to the database: {}", e);
            ApiError {
                err: "Database Error".to_string(),
                msg: Some(
                    "Cannot connect to the database, please contact the administrator".to_string(),
                ),
                http_status_code: Status::InternalServerError.code,
            }
        })?;
        let request_clone = request_clone_optional.ok_or(ApiError {
            err: "Database Error".to_string(),
            msg: Some("Unable to reproduce the request, you may try again immediately".to_string()),
            http_status_code: Status::ServiceUnavailable.code,
        })?;
        response = client.execute(request_clone).await.map_err(|e| {
            warn!(
                "Unable to execute the second request provided by the application: {}",
                e
            );
            request_error()
        })?;
        status = response.status();
    }
    if !status.is_success() {
        let couch_error = response.json::<DatabaseError>().await.map_err(|e| {
            warn!("Unable to return error to client: {}", e);
            request_error()
        })?;
        return Err(ApiError::from((couch_error, status)));
    }
    let deserialized_body = response.json::<R>().await.map_err(|e| {
        warn!("Unable to deserialize a response from the database: {}", e);
        request_error()
    })?;
    Ok(deserialized_body)
}

/// Checks whether the provided `id` is part of the `partition` ot not.
///
/// # Arguments
///
/// * `id`: the id to check
/// * `partition`: the partition which could contain the `id`
///
/// returns: Result<(), Error>
pub(crate) fn check_document_partition(id: &str, partition: &str) -> Result<(), ApiError> {
    if id.starts_with(format!("{}:", partition).as_str()) {
        Ok(())
    } else {
        Err(ApiError {
            err: "invalid id".to_string(),
            msg: Some("the provided id starts with an invalid partition".to_string()),
            http_status_code: Status::UnprocessableEntity.code,
        })
    }
}

/// Generate an id for a document with a given partition.
/// A UUID will be used, the format will be `partition:UUID`.
///
/// # Arguments
///
/// * `partition`: the partition to generate the id for
///
/// returns: String
pub(crate) fn generate_document_id(partition: &str) -> String {
    format!("{}:{}", partition, Uuid::new_v4())
}
