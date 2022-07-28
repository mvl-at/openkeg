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
use serde_json::{json, Value};

use crate::api_result::{Error, Result};
use crate::archive::database::{
    check_document_partition, fuzzy, generate_document_id, request, FindResponse,
    OperationResponse, Pagination,
};
use crate::archive::model::{Score, ScoreSearchTermField};
use crate::Config;

pub async fn all_scores(
    conf: &Config,
    client: &Client,
    limit: u64,
    skip: u64,
) -> Result<Pagination<Score>> {
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
    .map(|r| Json(r))
}

/// The service function to search for scores according to the given criteria.
/// All criteria are chained with the `$and` operator.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the client to send the requests with
/// * `search_term`: the term to search for, if `None` all `attributes` will be ignored
/// * `regex`: `Some(true)` if `search_term` should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
/// * `attributes`: the attributes to search for `search_term`
/// * `book`: the book to search for
/// * `location`: the location to search for
/// * `sort`: the attribute to define the order for
/// * `ascending`: `Some(false)` if the order should be descending, ascending otherwise
/// * `limit`: the limit of the amount of results to return
/// * `bookmark`: the bookmark using for pagination
///
/// returns: Result<Json<FindResponse<Score>>, Error>
pub async fn search_scores(
    conf: &Config,
    client: &Client,
    search_term: Option<String>,
    regex: Option<bool>,
    attributes: Vec<ScoreSearchTermField>,
    book: Option<String>,
    location: Option<String>,
    sort: Option<ScoreSearchTermField>,
    ascending: Option<bool>,
    limit: u64,
    bookmark: Option<String>,
) -> Result<FindResponse<Score>> {
    let filter = construct_filter(
        search_term,
        regex,
        attributes,
        book,
        location,
        sort,
        ascending,
        limit,
        bookmark,
    );
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
pub async fn get_score(conf: &Config, client: &Client, id: String) -> Result<Score> {
    let id_result = check_document_partition(&id, &conf.database.score_partition);
    if id_result.is_some() {
        return Err(id_result.unwrap());
    }
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
pub async fn put_score<'de>(conf: &Config, client: &Client, mut score: Score) -> Result<Score> {
    if (score.couch_id.is_none() && score.couch_revision.is_some())
        || (score.couch_id.is_some() && score.couch_revision.is_none())
    {
        return Err(Error {
            err: "invalid id".to_string(),
            msg: Some("you must either provide both id and rev, in order to update a document, or provide none of them, in order to insert one".to_string()),
            http_status_code: Status::BadRequest.code,
        });
    }
    let mut couch_id = score.couch_id.clone();
    if score.couch_id.is_some() {
        let id_result =
            check_document_partition(&couch_id.unwrap(), &conf.database.score_partition);
        if id_result.is_some() {
            return Err(id_result.unwrap());
        }
    } else {
        score.couch_id = Some(generate_document_id(&conf.database.score_partition));
    }
    couch_id = score.couch_id.clone();
    let api_url = format!(
        "{}/{}",
        conf.database.database_mapping.put_score,
        couch_id.unwrap()
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
) -> Result<OperationResponse> {
    let id_result = check_document_partition(&id, &conf.database.score_partition);
    if id_result.is_some() {
        return Err(id_result.unwrap());
    }
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
) -> Result<FindResponse<Score>> {
    let result = search_scores(
        conf,
        client,
        None,
        None,
        vec![],
        Some(book.clone()),
        None,
        None,
        None,
        0xffff,
        None,
    )
    .await;
    if result.is_err() {
        return result;
    }
    let mut response = result.unwrap();
    let scores = &mut response.docs;
    sort_by_book_page(&book, scores);
    Ok(response)
}

/// Construct a filter for the couchdb to search scores.
///
/// # Arguments
///
/// * `search_term`: the term to search for, if `None` all `attributes` will be ignored
/// * `regex`: `Some(true)` if `search_term` should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
/// * `attributes`: the attributes to search for `search_term`
/// * `book`: the book to search for
/// * `location`: the location to search for
/// * `sort`: the attribute to define the order for
/// * `ascending`: `Some(false)` if the order should be descending, ascending otherwise
/// * `limit`: the limit of the amount of results to return
/// * `bookmark`: the bookmark using for pagination
///
/// returns: Value
fn construct_filter(
    search_term: Option<String>,
    regex: Option<bool>,
    attributes: Vec<ScoreSearchTermField>,
    book: Option<String>,
    location: Option<String>,
    sort: Option<ScoreSearchTermField>,
    ascending: Option<bool>,
    limit: u64,
    bookmark: Option<String>,
) -> Value {
    let sort_value = sort
        .map(|s| json!({s.to_string().as_str(): ascending.unwrap_or(true)}))
        .unwrap_or(json!([]));
    let mut criteria = HashMap::new();
    if book.is_some() {
        let book_criteria = json!({"$elemMatch": {"book": book.unwrap()}});
        criteria.insert("pages".to_string(), book_criteria);
    }
    if location.is_some() {
        criteria.insert("location".to_string(), Value::String(location.unwrap()));
    }
    if search_term.is_some() {
        let term = search_term.unwrap();
        attributes.iter().for_each(|a| {
            let value = if a.is_array() {
                json!({
                        "$elemMatch": {
                            "$regex": term_from_regex(term.clone(), &regex)
                        }
                })
            } else {
                json!({
                        "$regex": term_from_regex(term.clone(), &regex)
                })
            };
            criteria.insert(a.to_string(), value);
        });
    }
    json!({
        "selector": json!(criteria),
        "sort": sort_value,
        "stable": true,
        "skip": 0,
        "execution_stats": true,
        "bookmark": bookmark,
        "limit": limit,
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
fn sort_by_book_page(book: &String, scores: &mut Vec<Score>) {
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
            return if page_opt_b.is_none() {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        }
        if page_opt_b.is_none() {
            return Ordering::Greater;
        }
        let page_a_begin = &page_opt_a.unwrap().begin;
        let page_b_begin = &page_opt_b.unwrap().begin;
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
