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

use reqwest::{Client, Method};
use rocket::serde::json::Json;
use serde_json::{json, Value};

use crate::archive::database::{request, FindResponse, Pagination};
use crate::archive::model::{Score, ScoreSearchTermField};
use crate::errors::Result;
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
/// * `search_term`: the term to search for, if `None` all [attributes] will be ignored
/// * `regex`: `Some(true)` if [search_term] should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
/// * `attributes`: the attributes to search for [search_term]
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
    .map(|r| Json(r))
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

/// Construct a filter for the couchdb to search scores.
///
/// # Arguments
///
/// * `search_term`: the term to search for, if `None` all [attributes] will be ignored
/// * `regex`: `Some(true)` if [search_term] should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
/// * `attributes`: the attributes to search for [search_term]
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
    let mut criteria: Vec<Value> = vec![];
    if book.is_some() {
        let book_criteria = json!({"pages":{"$elemMatch": {"book": book.unwrap()}}});
        criteria.push(book_criteria);
    }
    if location.is_some() {
        let location_criteria = json!({"location": location.unwrap()});
        criteria.push(location_criteria);
    }
    if search_term.is_some() {
        let term = search_term.unwrap();
        let mut attribute_criteria: Vec<Value> = attributes
            .iter()
            .map(|a| {
                if a.is_array() {
                    json!({
                        a.to_string().as_str(): {
                            "$elemMatch": {
                                "$regex": term_from_regex(term.clone(), &regex)
                            }
                        }
                    })
                } else {
                    json!({
                        a.to_string().as_str(): {
                            "$regex": term_from_regex(term.clone(), &regex)
                        }
                    })
                }
            })
            .collect();
        criteria.append(&mut attribute_criteria);
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
/// * `regex`: `Some(true)` if [search_term] should be interpreted as regex, otherwise it will be interpreted as a fuzzy search term
///
/// returns: String
fn term_from_regex(term: String, regex: &Option<bool>) -> String {
    if regex.unwrap_or(false) {
        term
    } else {
        fuzzy_regex(term)
    }
}

/// Convert the search term into a fuzzy one.
///
/// # Arguments
///
/// * `term`: the term to convert
///
/// returns: String
fn fuzzy_regex(term: String) -> String {
    term
}

fn no_op<'a, E>() -> Box<dyn FnOnce(E) -> E + Send + 'a> {
    Box::new(|e| e)
}
