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

use reqwest::Client;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::archive::model::Score;
use crate::database::client::{FindResponse, OperationResponse, Pagination};
use crate::database::score::{all_scores, ScoreSearchParameters};
use crate::openapi::ApiResult;
use crate::user::executives::{Archive, ExecutiveRole};
use crate::Config;

/// Get all scores from the database with pagination.
/// The parameters specify the value itself, the fields to search for and the ordering.
///
/// # Arguments
///
/// `limit`: the maximum amount of returned rows
/// `skip`: how many scores should be skipped
/// `conf`: the application configuration
/// `client`: the client to perform the database requests with
///
/// returns: ApiResult<Pagination<Score>>
#[openapi(tag = "Archive")]
#[get("/?<limit>&<skip>")]
pub async fn get_scores(
    limit: u64,
    skip: u64,
    _archive_role: ExecutiveRole<Archive>,
    conf: &State<Config>,
    client: &State<Client>,
) -> ApiResult<Pagination<Score>> {
    all_scores(conf, client, limit, skip).await
}

/// A request for searching scores in the database.
///
/// # Paginating
///
/// This request supports paginating in a quite inconvenient way:
/// One can only specify the `limit`, the offset/skip can be only described by the `bookmark`.
/// The `bookmark` works such as an iterator but with anchors.
/// E.g.: if `limit = 10` and `bookmark` is unset, the first 10 results will be shown.
/// Within this response, the server will return a `bookmark` string.
/// This string can be used in the next request in order to retrieve the next 10 results and so on.
///
/// # Arguments
///
/// * `parameters`: the parameters to perform the search
/// * `conf`: the application configuration
/// * `client`: the http client to perform the database query
///
/// returns: Result<Json<FindResponse<Score>>, Error>
#[openapi(tag = "Archive")]
#[get("/searches?<parameters..>")]
pub async fn search_scores(
    parameters: ScoreSearchParameters,
    _archive_role: ExecutiveRole<Archive>,
    conf: &State<Config>,
    client: &State<Client>,
) -> ApiResult<FindResponse<Score>> {
    crate::database::score::search_scores(conf, client, parameters).await
}

/// Find a single score by its id.
///
/// # Arguments
///
/// * `id`: the id of the document which contains the score
/// * `conf`: the application configuration
/// * `client` the client to send the request with
///
/// returns: Result<Json<Score>, Error>
#[openapi(tag = "Archive")]
#[get("/<id>")]
pub async fn get_score(
    id: String,
    _archive_role: ExecutiveRole<Archive>,
    conf: &State<Config>,
    client: &State<Client>,
) -> ApiResult<Score> {
    crate::database::score::get_score(conf, client, id).await
}

/// Insert a score into the database.
/// When creating a new score, make sure to leave its `_id` and `rev` to `None` and set both on update.
/// In the case of an `409 Conflict` just get the current revision of the score and try again.
///
/// # Arguments
///
/// * `score`: the score to insert
/// * `conf`: the application configuration
/// * `client`: the client to perform the request with
#[openapi(tag = "Archive")]
#[put("/", data = "<score>")]
pub async fn put_score(
    score: Json<Score>,
    _archive_role: ExecutiveRole<Archive>,
    conf: &State<Config>,
    client: &State<Client>,
) -> ApiResult<Score> {
    crate::database::score::put_score(conf, client, score.0).await
}

/// Delete a score by its id and revision.
///
/// # Arguments
///
/// * `id`: the id of the score to delete
/// * `rev`: the revision of the score to delete
/// * `conf`: the application configuration
/// * `client`: the client to perform the request
///
/// returns: Result<Json<OperationResponse>, Error>
#[openapi(tag = "Archive")]
#[delete("/<id>?<rev>")]
pub async fn delete_score(
    id: String,
    rev: String,
    _archive_role: ExecutiveRole<Archive>,
    conf: &State<Config>,
    client: &State<Client>,
) -> ApiResult<OperationResponse> {
    crate::database::score::delete_score(conf, client, id, rev).await
}
