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

use std::default::Default;

use okapi::map;
use okapi::openapi3::RefOr;
use okapi::openapi3::{Parameter, ParameterValue, Responses};
use rocket::fs::NamedFile;
use rocket::http::MediaType;
use rocket::request::FromParam;
use rocket::response::Responder;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::OpenApiFromParam;
use rocket_okapi::response::OpenApiResponderInner;
use schemars::schema::SchemaObject;
use serde_json::Value::String as Vs;

use crate::config::DocumentMapping;

/// The type of a document such as blackboard or chronicle.
/// Used to differentiate between types of a document and their origin.
pub enum DocumentType {
    /// A short document for posting new such as a big event.
    Blackboard,
}

const BLACKBOARD_ID: &str = "blackboard";

impl DocumentType {
    /// Get the location to the directory where all the documents of this type are stored at.
    /// The returned path is relative to the document server.
    ///
    /// # Arguments
    ///
    /// * `mapping`: the mapping from the configuration
    ///
    /// returns: String
    pub fn location(&self, mapping: &DocumentMapping) -> String {
        match self {
            DocumentType::Blackboard => mapping.blackboard.clone(),
        }
    }

    /// Get the location to the assets directory where all the document-assets of this type are stored at.
    /// The returned path is relative to the document server.
    ///
    /// # Arguments
    ///
    /// * `mapping`: the mapping from the configuration
    ///
    /// returns: String
    pub fn assets_location(&self, mapping: &DocumentMapping) -> String {
        match self {
            DocumentType::Blackboard => mapping.blackboard_assets.clone(),
        }
    }
}

impl ToString for DocumentType {
    fn to_string(&self) -> String {
        match self {
            DocumentType::Blackboard => BLACKBOARD_ID,
        }
        .to_string()
    }
}

impl FromParam<'_> for DocumentType {
    type Error = ();

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        match param {
            BLACKBOARD_ID => Ok(DocumentType::Blackboard),
            _ => Err(()),
        }
    }
}

impl OpenApiFromParam<'_> for DocumentType {
    fn path_parameter(
        _gen: &mut OpenApiGenerator,
        name: String,
    ) -> rocket_okapi::Result<Parameter> {
        Ok(Parameter {
            name,
            location: "path".to_string(),
            description: Some("The type of the document".to_string()),
            required: true,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema: SchemaObject {
                    metadata: None,
                    instance_type: None,
                    format: Some("string".to_string()),
                    enum_values: Some(vec![Vs(DocumentType::Blackboard.to_string())]),
                    const_value: None,
                    subschemas: None,
                    number: None,
                    string: None,
                    array: None,
                    object: None,
                    reference: None,
                    extensions: Default::default(),
                },
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        })
    }
}

#[derive(Responder)]
#[response(status = 200, content_type = "text/markdown")]
pub struct MarkdownContent(pub NamedFile);

//todo: find a less verbose way to propagate text/markdown to openapi
impl OpenApiResponderInner for MarkdownContent {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let markdown = okapi::openapi3::MediaType::default();
        let markdown_response = okapi::openapi3::Response {
            description: "The document in the markdown format".to_string(),
            content: map! {MediaType::Markdown.to_string() => markdown},
            ..okapi::openapi3::Response::default()
        };
        let responses = map! {"200".to_string() => RefOr::Object(markdown_response)};
        Ok(Responses {
            default: None,
            responses,
            extensions: map! {},
        })
    }
}
