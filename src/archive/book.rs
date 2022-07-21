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

use crate::archive::database::Pagination;
use crate::archive::model::Score;
use crate::errors::Result;
use crate::schema_util::SchemaExample;

/// Return the scores of a book in the correct order by their page.
#[openapi(tag = "Archive")]
#[get("/<name>/content")]
pub fn get_book_content(name: String) -> Result<Pagination<Score>> {
    Ok(Json(SchemaExample::example()))
}
