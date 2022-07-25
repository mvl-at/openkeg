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

use crate::archive::database::{request, Pagination};
use crate::archive::model::Score;
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
