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

use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::archive::model::{Page, Score, ScoreSearchParameters};
use crate::errors::Result;
use crate::schema_util::SchemaExample;
use crate::{schema_util, Config};

/// Search for scores which fulfil the passed parameters.
/// The parameters specify the value itself, the fields to search for and the ordering.
#[openapi(tag = "Archive")]
#[get("/score?<params..>")]
pub fn search_scores(params: ScoreSearchParameters) -> Result<schema_util::Page<Score>> {
    Ok(Json(schema_util::Page::example()))
}

/// Return a single score.
#[openapi(tag = "Archive")]
#[get("/score/<id>")]
pub fn get_score(id: i64) -> Result<Score> {
    Ok(Json(Score::example()))
}

/// Create or update a score.
#[openapi(tag = "Archive")]
#[put("/score", data = "<score>")]
pub fn put_score(mut score: Json<Score>, conf: &State<Config>) -> Result<Score> {
    Ok(Json(Score::example()))
}

/// Delete a score by its id.
#[openapi(tag = "Archive")]
#[delete("/score/<id>")]
pub fn delete_score(id: i64) -> Result<()> {
    Ok(Json(()))
}

/// Return the pages of a book in the correct order by their id.
#[openapi(tag = "Archive")]
#[get("/book/<id>/content")]
pub fn get_book_content(id: i64) -> Result<Vec<Page>> {
    Ok(Json(vec![Page::example()]))
}
