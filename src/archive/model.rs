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

use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::fmt;

use crate::schema_util::SchemaExample;

/// Representation of a score considering the intellectual property, not a specific copy.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", default)]
#[schemars(example = "Self::example")]
pub struct Score {
    /// The id of the score which couch db is using
    #[serde(rename = "_id")]
    pub couch_id: Option<String>,
    /// The revision of the document couch db is using
    #[serde(rename = "_rev")]
    pub couch_revision: Option<String>,
    /// The legacy id of this score. Will be removed.
    #[deprecated]
    pub legacy_ids: Vec<i64>,
    /// The main title of this score.
    pub title: String,
    /// The genres of the score.
    pub genres: Vec<String>,
    /// All composers of the score.
    /// The order is not considered here and every composer will only be persisted once per score and over the whole database.
    pub composers: Vec<String>,
    /// All arrangers of the score.
    /// The order is not considered here and every arranger will only be persisted once per score and over the whole database.
    pub arrangers: Vec<String>,
    /// All publishers of the score.
    /// The order is not considered here and every publisher will only be persisted once per score and over the whole database.
    pub publisher: Option<String>,
    /// The grade of this score.
    pub grade: Option<String>,
    /// Other known titles for the scores.
    /// Often bohemian titles.
    /// The order is not considered here and every alias will only be persisted once.
    pub alias: Vec<String>,
    /// The subtitles of the score which can be used to mark smes, potpourri or medley.
    /// The order is mandatory here and every sub-title can occur more than once.
    pub subtitles: Vec<String>,
    /// The annotation of this score.
    pub annotation: Option<String>,
    /// The score of the back of this one.
    pub back_title: Option<String>,
    /// Where the score currently is located at.
    pub location: Option<String>,
    /// Indicates whether this score has a conductor score or not
    pub conductor_score: bool,
    /// The pages where this score is located at.
    pub pages: Vec<Page>,
}

/// A book which contains scores as pages.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct Book {
    /// The id of the book.
    #[serde(skip_deserializing)]
    pub id: Option<i64>,
    /// Full name of the book.
    pub name: String,
    /// Annotation of the book.
    pub annotation: Option<String>,
}

/// A page which represents where a particular score is located in a book.
/// A page can only contain one score at maximum.
/// When a page contains multiple scores, only the first one will be stored here.
/// The other scores should be persisted via references.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct Page {
    /// The title of the book where this page is located at.
    pub book: String,
    /// The number where the page begins at.
    pub begin: PageNumber,
    /// The number where the page ends at.
    /// The page ends at `begin` if absent.
    pub end: Option<PageNumber>,
}

/// A representation of a page during its insert or update in the context of a book.
/// The basic rules of [Page] apply here too as this is only a different representation.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct PagePlacement {
    /// The number where the page begins at.
    pub begin: PageNumber,
    /// The number where the page ends at.
    /// The page ends at `begin` if absent.
    pub end: Option<PageNumber>,
    /// The id of the [Score] which is places at this page.
    pub score: i64,
}

/// A page-number.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct PageNumber {
    /// The prefix of the page.
    pub prefix: Option<String>,
    /// The actual number of the page.
    pub number: Option<i64>,
    /// The suffix of the page.
    pub suffix: Option<String>,
}

/// Search parameters for a score search.
/// This consists of a search-term, the fields to search for,
/// whether the results should be ascending or not and the order of the results.
#[derive(JsonSchema, FromForm, Debug)]
pub struct ScoreSearchParameters {
    /// The search term.
    pub term: String,
    /// The fields where to search.
    pub fields: Vec<ScoreSearchTermField>,
    /// `true` if the results should be ordered ascending, `false` otherwise.
    pub ascending: Option<bool>,
    /// The field which specifies the ordering of the results.
    pub order: Option<ScoreSearchTermField>,
}

/// Representation of a score field which can be used in a search.
#[derive(Debug, Serialize, Deserialize, JsonSchema, FromFormField)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub enum ScoreSearchTermField {
    Title,
    Genres,
    Subtitles,
    Arrangers,
    Composers,
    Alias,
    Publisher,
}

impl fmt::Display for ScoreSearchTermField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type CountStatistic = Statistic<String, u64>;

/// A statistic from the database.
/// Typically the result of reduced design documents.
#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct Statistic<K, V> {
    /// The rows of the statistic.
    pub rows: Vec<StatisticEntry<K, V>>,
}

/// A single statistic entry which may contain information such as a count as a value for an string id.
#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct StatisticEntry<K, V> {
    /// The key of this statistic entry.
    pub key: K,
    /// The value of this statistic entry.
    pub value: V,
}

impl SchemaExample for Score {
    #[allow(deprecated)]
    fn example() -> Self {
        Self {
            couch_id: None,
            couch_revision: None,
            legacy_ids: vec![],
            title: "baum".to_string(),
            genres: vec![],
            composers: vec![],
            arrangers: vec![],
            publisher: Some("Hansl Verlag".to_string()),
            grade: None,
            alias: vec!["strauch".to_string(), "teller".to_string()],
            subtitles: vec![],
            annotation: None,
            back_title: None,
            location: None,
            conductor_score: false,
            pages: vec![],
        }
    }
}

impl SchemaExample for PageNumber {
    fn example() -> Self {
        Self {
            prefix: Some("A".to_string()),
            number: Some(6),
            suffix: None,
        }
    }
}

impl SchemaExample for Page {
    fn example() -> Self {
        Self {
            book: "Rot".to_string(),
            begin: Default::default(),
            end: None,
        }
    }
}

impl SchemaExample for PagePlacement {
    fn example() -> Self {
        Self {
            score: 3,
            begin: Default::default(),
            end: Default::default(),
        }
    }
}

impl SchemaExample for Book {
    fn example() -> Self {
        Self {
            id: Some(5),
            name: "Rot".to_string(),
            annotation: Some("New covers".to_string()),
        }
    }
}

impl SchemaExample for ScoreSearchTermField {
    fn example() -> Self {
        Self::Title
    }
}

impl ScoreSearchTermField {
    pub fn is_array(&self) -> bool {
        match self {
            ScoreSearchTermField::Title => false,
            ScoreSearchTermField::Genres => true,
            ScoreSearchTermField::Subtitles => true,
            ScoreSearchTermField::Arrangers => true,
            ScoreSearchTermField::Composers => true,
            ScoreSearchTermField::Alias => true,
            ScoreSearchTermField::Publisher => false,
        }
    }
}
