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

use crate::api_result::Result;
use crate::archive::database;
use crate::archive::database::score::all_scores;
use crate::archive::database::{FindResponse, OperationResponse, Pagination};
use crate::archive::model::{Score, ScoreSearchTermField};
use crate::schema_util::SchemaExample;
use crate::Config;

/// Get all scores from the database with pagination.
/// The parameters specify the value itself, the fields to search for and the ordering.
#[openapi(tag = "Archive")]
#[get("/?<limit>&<skip>")]
pub async fn get_scores(
    limit: u64,
    skip: u64,
    conf: &State<Config>,
    client: &State<Client>,
) -> Result<Pagination<Score>> {
    all_scores(conf, client, limit, skip).await
}

/// A request for searching scores in the database.
///
/// # Paginating
///
/// This request supports paginating in a quite inconvenient way:
/// One can only specify the [limit], the offset/skip can be only described by the [bookmark].
/// The [bookmark] works such as an iterator but with anchors.
/// E.g.: if `limit = 10` and `bookmark` is unset, the first 10 results will be shown.
/// Within this response, the server will return a `bookmark` string.
/// This string can be used in the next request in order to retrieve the next 10 results and so on.
///
/// # Arguments
///
/// * `search_term`: a string to search for in the specified [attributes]
/// * `regex`: if `true` the [search_term] will be interpreted as a regular expression instead of a fuzzy search term
/// * `attributes`: the attributes to search for
/// * `book`: if set, the score must contain a page with exactly this book
/// * `location`: if set, the score must be have set a location with exact this string
/// * `sort`: the field which should be used to sort the results (database relative, not page)
/// * `ascending`: is unset or `true` the results will be sorted ascending, descending otherwise
/// * `limit`: the limit of documents for a result page
/// * `bookmark`: the bookmark used for pagination
///
/// returns: Result<Json<FindResponse<Score>>, Error>
#[openapi(tag = "Archive")]
#[get(
"/searches?<search_term>&<regex>&<attributes>&<book>&<location>&<sort>&<ascending>&<limit>&<bookmark>"
)]
pub async fn search_scores(
    search_term: Option<String>,
    regex: Option<bool>,
    attributes: Vec<ScoreSearchTermField>,
    book: Option<String>,
    location: Option<String>,
    sort: Option<ScoreSearchTermField>,
    ascending: Option<bool>,
    limit: u64,
    bookmark: Option<String>,
    conf: &State<Config>,
    client: &State<Client>,
) -> Result<FindResponse<Score>> {
    database::score::search_scores(
        conf,
        client,
        search_term,
        regex,
        attributes,
        book,
        location,
        sort,
        ascending,
        limit,
        bookmark,
    )
    .await
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
pub async fn get_score(id: String, conf: &State<Config>, client: &State<Client>) -> Result<Score> {
    database::score::get_score(conf, client, id).await
}

/// Create or update a score.
#[openapi(tag = "Archive")]
#[put("/", data = "<score>")]
pub fn put_score(score: Json<Score>, conf: &State<Config>) -> Result<Score> {
    Ok(Json(Score::example()))
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
    conf: &State<Config>,
    client: &State<Client>,
) -> Result<OperationResponse> {
    database::score::delete_score(conf, client, id, rev).await
}
