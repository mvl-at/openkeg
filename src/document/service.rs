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

use std::path::Path;

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::document::model::{DocumentType, MarkdownContent};
use crate::openapi::{map_io_err, ApiError, ApiResult};
use crate::Config;

/// List all documents of the provided [`DocumentType`] which are available on the server sorted by their filename.
/// The list only includes files directly located at the configured directory of the document type.
/// This means there is no support for recursive lookups nor directories.
///
/// # Arguments
///
/// * `doc_type`: the document type of all the listed documents
/// * `conf`: the application configuration
///
/// returns: Result<Json<Vec<String, Global>>, ApiError>
#[openapi(tag = "Documents")]
#[get("/<doc_type>")]
pub async fn list_documents(
    doc_type: DocumentType,
    conf: &State<Config>,
) -> ApiResult<Vec<String>> {
    let doc_type_path_str = doc_type.location(&conf.document_server.mapping);
    let doc_type_path = map_io_err(
        Path::new(&doc_type_path_str).canonicalize(),
        Status::InternalServerError,
    )?;
    let read_dir = map_io_err(doc_type_path.read_dir(), Status::InternalServerError)?;
    let files = read_dir.flatten().filter(|f| f.path().is_file());
    let mut files_names: Vec<String> = files
        .flat_map(|f| f.file_name().to_str().map(|s| s.to_string()))
        .collect();
    files_names.sort();
    Ok(Json(files_names))
}

/// Read a document located on the servers file system.
/// Each document has a [DocumentType] with a corresponding base url.
/// If the requested document name is not below the location of the [DocumentType], the server will return a 'Not Found'.
///
/// # Arguments
///
/// * `doc_type`: the document type to look for
/// * `document`: the file name of the requested document
/// * `conf`: the application configuration
///
/// returns: Result<MarkdownContent, ApiError>
#[openapi(tag = "Documents")]
#[get("/<doc_type>/<document>", format = "text/markdown")]
pub async fn get_document(
    doc_type: DocumentType,
    document: String,
    conf: &State<Config>,
) -> Result<MarkdownContent, ApiError> {
    let doc_type_path_str = doc_type.location(&conf.document_server.mapping);
    let doc_type_path = map_io_err(
        Path::new(&doc_type_path_str).canonicalize(),
        Status::InternalServerError,
    )?;
    let path = map_io_err(
        doc_type_path.join(document).canonicalize(),
        Status::UnprocessableEntity,
    )?;
    if !path.as_path().starts_with(doc_type_path) {
        return Err(ApiError {
            err: "Not Found".to_string(),
            msg: Some("File or directory not found".to_string()),
            http_status_code: Status::NotFound.code,
        });
    }
    let doc = map_io_err(NamedFile::open(path).await, Status::NotFound)?;
    Ok(MarkdownContent(doc))
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
