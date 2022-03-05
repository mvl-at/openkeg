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

use rocket::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Trait for adds examples to API documentation
pub trait SchemaExample {
    fn example() -> Self;
}

/// A page for pagination which is used for huge collections as the score archive.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct Page<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The maximum amount of items returned per page.
    /// Normally taken from the original request.
    pub limit: u64,
    /// The size of the results vector.
    pub size: u64,
    /// The offset where to begin to query.
    /// Starts with 0.
    pub offset: u64,
    /// The total amount of items contained in the whole collection.
    pub total: u64,
    /// The actual results.
    /// Will be empty when `offset >= length`.
    pub results: Vec<D>,
}

impl<D> SchemaExample for Page<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            limit: 20,
            size: 0,
            offset: 150,
            total: 150,
            results: vec![],
        }
    }
}
