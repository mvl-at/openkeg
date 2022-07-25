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

use std::io::Cursor;

use okapi::map;
use okapi::openapi3::{RefOr, Responses};
use rocket::http::{ContentType, MediaType};
use rocket::response::Responder;
use rocket::{Request, Response};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;

pub struct Photo(pub(crate) Vec<u8>);

impl<'r> Responder<'r, 'static> for Photo {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .header(ContentType::JPEG)
            .streamed_body(Cursor::new(self.0))
            .ok()
    }
}

impl<'r> OpenApiResponderInner for Photo {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let jpeg = okapi::openapi3::MediaType::default();
        let image_response = okapi::openapi3::Response {
            description: "The photo image of the member".to_string(),
            content: map! {MediaType::JPEG.to_string() => jpeg},
            ..okapi::openapi3::Response::default()
        };
        let responses = map! {"200".to_string() => RefOr::Object(image_response)};
        Ok(Responses {
            default: None,
            responses,
            extensions: map! {},
        })
    }
}
