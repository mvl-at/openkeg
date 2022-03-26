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
use rocket_okapi::openapi;

use crate::archive::model::{Book, Page, PageNumber, PagePlacement, Score, ScoreSearchParameters};
use crate::errors::Result;
use crate::schema_util;
use crate::schema_util::SchemaExample;

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

/// Update an already existing score by its id.
#[openapi(tag = "Archive")]
#[put("/score/<id>", data = "<score>")]
pub fn put_score(id: i64, score: Json<Score>) -> Result<Score> {
    Ok(Json(Score::example()))
}

/// Delete a score by its id.
#[openapi(tag = "Archive")]
#[delete("/score/<id>")]
pub fn delete_score(id: i64) -> Result<()> {
    Ok(Json(()))
}

/// Return a book by its id.
#[openapi(tag = "Archive")]
#[get("/book/<id>")]
pub fn get_book(id: i64) -> Result<Book> {
    Ok(Json(Book::example()))
}

/// Create a new book.
#[openapi(tag = "Archive")]
#[post("/book", data = "<book>")]
pub fn post_book(book: Json<Book>) -> Result<Book> {
    Ok(Json(Book::example()))
}

/// Update an already existing book by its id.
#[openapi(tag = "Archive")]
#[put("/book", data = "<book>")]
pub fn put_book(book: Json<Book>) -> Result<Book> {
    Ok(Json(Book::example()))
}

/// Delete a book by its id.
#[openapi(tag = "Archive")]
#[delete("/book/<id>")]
pub fn delete_book(id: i64) -> Result<()> {
    Ok(Json(()))
}

/// Return the pages of a book in the correct order by their id.
#[openapi(tag = "Archive")]
#[get("/book/<id>/content")]
pub fn get_book_content(id: i64) -> Result<Vec<Page>> {
    Ok(Json(vec![Page::example()]))
}

/// Place a score at a page in a book.
/// This will create a whole new page if necessary when no page with `begin` of this page exists in this book.
/// All non identifying attributes will be overwritten in the persistence such as the page `end` as well as the score.
#[openapi(tag = "Archive")]
#[put("/book/<id>/page", data = "<page>")]
pub fn put_book_page(id: i64, page: Json<PagePlacement>) -> Result<Page> {
    Ok(Json(Page::example()))
}

/// Delete a page entry from a book.
/// The score itself will remain in the persistence.
#[openapi(tag = "Archive")]
#[delete("/book/<id>/page", data = "<page>")]
pub fn delete_book_page(id: i64, page: Json<PageNumber>) -> Result<()> {
    Ok(Json(()))
}
