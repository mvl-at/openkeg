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

use std::cmp::Ordering;
use std::collections::HashMap;

use reqwest::{Client, Method};
use rocket::http::Status;
use rocket::serde::json::Json;
use schemars::JsonSchema;
use serde_json::{json, Value};

use crate::archive::model::{Score, ScoreSearchTermField};
use crate::database::client::{
    check_document_partition, generate_document_id, request, FindResponse, OperationResponse,
    Pagination,
};
use crate::database::fuzzy;
use crate::openapi::{ApiError, ApiResult};
use crate::Config;

pub async fn all_scores(
    conf: &Config,
    client: &Client,
    limit: u64,
    skip: u64,
) -> ApiResult<Pagination<Score>> {
    let mut parameters = HashMap::new();
    parameters.insert("include_docs".to_string(), "true".to_string());
    parameters.insert("limit".to_string(), limit.to_string());
    parameters.insert("skip".to_string(), skip.to_string());
    request(
        conf,
        client,
        Box::new(|r| r),
        Method::GET,
        &conf.database.database_mapping.all_scores,
        &parameters,
    )
    .await
    .map(Json)
}

/// The parameters used to search scores.
#[derive(FromForm, JsonSchema)]
pub struct ScoreSearchParameters {
    /// A string to search for in the specified `attributes`.
    search_term: Option<String>,
    /// If `true` the `search_term` will be interpreted as a regular expression instead of a fuzzy search term.
    regex: Option<bool>,
    /// The attributes to search for.
    attributes: Vec<ScoreSearchTermField>,
    /// If set, the score must contain a page with exactly this book.
    book: Option<String>,
    /// If set, the score must be have set a location with exact this string.
    location: Option<String>,
    /// The field which should be used to sort the results (database relative, not page).
    sort: Option<ScoreSearchTermField>,
    /// If unset or `true` the results will be sorted ascending, descending otherwise.
    ascending: Option<bool>,
    /// The limit of documents for a result page.
    limit: u64,
    /// The bookmark used for pagination.
    bookmark: Option<String>,
}

/// The service function to search for scores according to the given criteria.
/// All criteria are chained with the `$and` operator.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the client to send the requests with
/// * `parameters`: the parameters to perform the search
///
/// returns: Result<Json<FindResponse<Score>>, Error>
pub async fn search_scores(
    conf: &Config,
    client: &Client,
    parameters: ScoreSearchParameters,
) -> ApiResult<FindResponse<Score>> {
    let filter = construct_filter(parameters);
    debug!("Using filter to search scores: {}", filter);
    let parameters: HashMap<String, String> = HashMap::new();
    request(
        conf,
        client,
        Box::new(|r| r.json(&filter)),
        Method::POST,
        &conf.database.database_mapping.find_scores,
        &parameters,
    )
    .await
    .map(Json)
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
pub async fn get_score(conf: &Config, client: &Client, id: String) -> ApiResult<Score> {
    check_document_partition(&id, &conf.database.score_partition)?;
    let parameters: HashMap<String, String> = HashMap::new();
    request(
        conf,
        client,
        no_op(),
        Method::GET,
        &format!("{}/{}", &conf.database.database_mapping.get_score, id),
        &parameters,
    )
    .await
    .map(Json)
}

/// Insert a score into the database.
/// When creating a new score, make sure to leave its `_id` and `rev` to `None` and set both on update.
/// In the case of an `409 Conflict` just get the current revision of the score and try again.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the client to perform the request with
/// * `score`: the score to insert
pub async fn put_score<'de>(conf: &Config, client: &Client, mut score: Score) -> ApiResult<Score> {
    if (score.couch_id.is_none() && score.couch_revision.is_some())
        || (score.couch_id.is_some() && score.couch_revision.is_none())
    {
        return Err(ApiError {
            err: "invalid id".to_string(),
            msg: Some("you must either provide both id and rev, in order to update a document, or provide none of them, in order to insert one".to_string()),
            http_status_code: Status::BadRequest.code,
        });
    }
    if let Some(couch_id) = &score.couch_id {
        check_document_partition(couch_id, &conf.database.score_partition)?;
    } else {
        score.couch_id = Some(generate_document_id(&conf.database.score_partition));
    }
    let api_url = format!(
        "{}/{}",
        conf.database.database_mapping.put_score,
        score
            .couch_id
            .as_ref()
            .expect("Checked or generated score id")
    );
    let parameters: HashMap<String, String> = HashMap::new();
    request(
        conf,
        client,
        Box::new(|r| r.json(&score)),
        Method::PUT,
        &api_url,
        &parameters,
    )
    .await
    .map(Json)
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
pub async fn delete_score(
    conf: &Config,
    client: &Client,
    id: String,
    rev: String,
) -> ApiResult<OperationResponse> {
    check_document_partition(&id, &conf.database.score_partition)?;
    let mut parameters: HashMap<String, String> = HashMap::new();
    parameters.insert("rev".to_string(), rev);
    request(
        conf,
        client,
        no_op(),
        Method::DELETE,
        &format!("{}/{}", &conf.database.database_mapping.delete_score, id),
        &parameters,
    )
    .await
    .map(Json)
}

/// Fetch all scores which are part of the given `book`.
/// The scores are sorted as usual in books which means the following order:
///
/// . `prefix` (`None` last)
/// . `number`
/// . `suffix` (`None` last)
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the client to send the database requests with
/// * `name`: the name of the book to fetch
///
/// returns: Result<Json<FindResponse<Score>>, Error>
pub async fn get_book_content(
    conf: &Config,
    client: &Client,
    book: String,
) -> ApiResult<FindResponse<Score>> {
    let mut response = search_scores(
        conf,
        client,
        ScoreSearchParameters {
            search_term: None,
            regex: None,
            attributes: vec![],
            book: Some(book.clone()),
            location: None,
            sort: None,
            ascending: None,
            limit: 0xffff,
            bookmark: None,
        },
    )
    .await?;
    let scores = &mut response.docs;
    sort_by_book_page(&book, scores);
    Ok(response)
}

/// Construct a filter for the couchdb to search scores.
///
/// # Arguments
///
/// * `parameters`: the parameters to construct the json value filter for
///
/// returns: Value
fn construct_filter(parameters: ScoreSearchParameters) -> Value {
    let sort_value = parameters.sort.map(|s| json!([{s.to_string().to_lowercase().as_str(): if parameters.ascending.unwrap_or(true) {"asc"} else {"desc}"}}])).unwrap_or(json!([]));
    let mut and_criteria = HashMap::new();
    let mut search_term_criteria = vec![];
    if let Some(book) = parameters.book {
        let book_criteria = json!({"$elemMatch": {"book": book}});
        and_criteria.insert("pages".to_string(), book_criteria);
    }
    if let Some(l) = parameters.location {
        and_criteria.insert("location".to_string(), Value::String(l));
    }
    if let Some(term) = parameters.search_term {
        parameters.attributes.iter().for_each(|a| {
            let key = a.to_string().to_lowercase();
            let value = if a.is_array() {
                json!({key: {
                        "$elemMatch": {
                            "$regex": term_from_regex(term.clone(), &parameters.regex)
                        }
                    }
                })
            } else {
                json!({key: {
                        "$regex": term_from_regex(term.clone(), &parameters.regex)
                }})
            };
            search_term_criteria.push(value);
        });
        and_criteria.insert("$or".to_string(), json!(search_term_criteria));
    }
    json!({
        "selector": json!(and_criteria),
        "sort": sort_value,
        "stable": true,
        "skip": 0,
        "execution_stats": true,
        "bookmark": parameters.bookmark,
        "limit": parameters.limit,
    })
}

/// Convenient function to convert the search term into a fuzzy one.
///
/// # Arguments
///
/// * `term`: the term to convert
/// * `regex`: `Some(true)` if `search_term` should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
///
/// returns: String
fn term_from_regex(term: String, regex: &Option<bool>) -> String {
    if regex.unwrap_or(false) {
        term
    } else {
        fuzzy::fuzzy_regex(term)
    }
}

fn no_op<'a, E>() -> Box<dyn FnOnce(E) -> E + Send + 'a> {
    Box::new(|e| e)
}

/// Function to in-place sort scores as in books.
/// This requires a book name and the scores to sort.
/// Scores will be sorted by their pages in the following order:
///
/// . `prefix` (`None` last)
/// . `number`
/// . `suffix` (`None` last)
///
/// # Arguments
///
/// * `book`: the book to use for the pages
/// * `scores`: the scores to sort
fn sort_by_book_page(book: &str, scores: &mut Vec<Score>) {
    scores.sort_by(|score_a, score_b| {
        let page_opt_a = score_a
            .pages
            .iter()
            .find(|p| book.eq_ignore_ascii_case(p.book.as_str()));
        let page_opt_b = score_b
            .pages
            .iter()
            .find(|p| book.eq_ignore_ascii_case(p.book.as_str()));
        if page_opt_a.is_none() {
            return page_opt_b.map_or_else(|| Ordering::Equal, |_| Ordering::Less);
        }
        if page_opt_b.is_none() {
            return Ordering::Greater;
        }
        let page_a_begin = &page_opt_a.expect("page of score_a").begin;
        let page_b_begin = &page_opt_b.expect("page of score_b").begin;
        let prefix_ordering = page_a_begin.prefix.cmp(&page_b_begin.prefix);
        if prefix_ordering != Ordering::Equal {
            return if page_a_begin.prefix.is_none() || page_b_begin.prefix.is_none() {
                prefix_ordering.reverse()
            } else {
                prefix_ordering
            };
        }
        let number_ordering = page_a_begin.number.cmp(&page_b_begin.number);
        if number_ordering != Ordering::Equal {
            return number_ordering;
        }
        let suffix_ordering = page_a_begin.suffix.cmp(&page_b_begin.suffix);
        if page_a_begin.suffix.is_none() || page_b_begin.suffix.is_none() {
            suffix_ordering.reverse()
        } else {
            suffix_ordering
        }
    });
}
