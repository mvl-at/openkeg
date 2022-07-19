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
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example", title = "ResultPage")]
pub struct Page<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The size of the results vector.
    pub total_rows: u64,
    /// The offset where to begin to query.
    /// Starts with 0.
    pub offset: u64,
    /// The actual results.
    /// Will be empty when `offset >= total_rows`.
    pub rows: Vec<D>,
}

impl<D> SchemaExample for Page<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            total_rows: 150,
            offset: 150,
            rows: vec![],
        }
    }
}

/// A page for pagination which is used for huge collections as the score archive.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example", title = "ResultRow")]
pub struct Row<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    /// The emitted id of the row
    pub id: String,
    /// The emitted key of the row
    pub key: String,
    /// The actual document of this row
    pub doc: D,
}

impl<D> SchemaExample for Row<D>
where
    D: Serialize + JsonSchema + SchemaExample,
{
    fn example() -> Self {
        Self {
            id: "score:289j9f84".to_string(),
            key: "score:289j9f84".to_string(),
            doc: SchemaExample::example(),
        }
    }
}
