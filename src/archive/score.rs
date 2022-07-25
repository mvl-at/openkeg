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

use crate::archive::database::score::all_scores;
use crate::archive::database::Pagination;
use crate::archive::model::Score;
use crate::errors::Result;
use crate::schema_util::SchemaExample;
use crate::Config;

/// Search for scores which fulfil the passed parameters.
/// The parameters specify the value itself, the fields to search for and the ordering.
#[openapi(tag = "Archive")]
#[get("/?<limit>&<skip>")]
pub async fn search_scores(
    limit: u64,
    skip: u64,
    conf: &State<Config>,
    client: &State<Client>,
) -> Result<Pagination<Score>> {
    all_scores(conf, client, limit, skip).await
}

/// Return a single score.
#[openapi(tag = "Archive")]
#[get("/<id>")]
pub fn get_score(id: i64) -> Result<Score> {
    Ok(Json(Score::example()))
}

/// Create or update a score.
#[openapi(tag = "Archive")]
#[put("/", data = "<score>")]
pub fn put_score(score: Json<Score>, conf: &State<Config>) -> Result<Score> {
    Ok(Json(Score::example()))
}

/// Delete a score by its id.
#[openapi(tag = "Archive")]
#[delete("/<id>")]
pub fn delete_score(id: i64) -> Result<()> {
    Ok(Json(()))
}
