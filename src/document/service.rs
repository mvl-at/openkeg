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

use rocket::State;
use rocket_okapi::openapi;

use crate::document::model::{DocumentType, MarkdownContent};
use crate::Config;

#[openapi(tag = "Documents")]
#[get("/<doc_type>")]
pub async fn list_documents(doc_type: DocumentType, conf: &State<Config>) {}

#[openapi(tag = "Documents")]
#[get("/<doc_type>/<document>", format = "text/markdown")]
pub async fn get_document(
    doc_type: DocumentType,
    document: String,
    conf: &State<Config>,
) -> MarkdownContent {
    MarkdownContent("baum".to_string())
}

#[openapi(tag = "Documents")]
#[get("/<doc_type>/<document>/assets/<asset>")]
pub async fn get_asset(
    doc_type: DocumentType,
    document: String,
    asset: String,
    conf: &State<Config>,
) {
}
