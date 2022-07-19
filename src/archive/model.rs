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
use rocket_okapi::JsonSchema;

use crate::schema_util::SchemaExample;

/// Representation of a score considering the intellectual property, not a specific copy.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[schemars(example = "Self::example")]
pub struct Score {
    /// The id of the score which couch db is using
    #[serde(rename = "_id")]
    pub couch_id: Option<String>,
    /// The revision of the document couch db is using
    #[serde(rename = "_rev")]
    pub couch_revision: Option<String>,
    /// The id of the score, should be generated by the dbms.
    pub id: Option<i64>,
    /// The legacy id of this score. Will be removed.
    #[deprecated]
    pub legacy_id: Option<i64>,
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
    /// The id of the main publisher. Will be removed.
    #[deprecated]
    pub publisher_id: Option<i64>,
    /// The grade of this score.
    pub grade: Option<String>,
    /// The grade id of this score. Will be removed.
    #[deprecated]
    pub grade_id: Option<i64>,
    /// Other known titles for the scores.
    /// Often bohemian titles.
    /// The order is not considered here and every alias will only be persisted once.
    pub alias: Vec<String>,
    /// The subtitles of the score which can be used to mark smes, potpourri or medley.
    /// The order is mandatory here and every sub-title can occur more than once.
    pub subtitles: Vec<String>,
    /// The annotation of this score.
    pub annotation: Option<String>,
    /// The id of the score of the back this one.
    pub back_of_id: Option<i64>,
    /// The score of the back of this one.
    pub back_of: Option<i64>,
    /// Where the score currently is located at.
    pub location: Option<String>,
    /// The legacy field of the location. Will be removed.
    #[deprecated]
    pub legacy_location: Option<String>,
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
    /// The page ends at [begin] if absent.
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
    pub fields: Vec<ScoreSearchParameter>,
    /// `true` if the results should be ordered ascending, `false` otherwise.
    pub ascending: Option<bool>,
    /// The field which specifies the ordering of the results.
    pub order: Option<ScoreSearchParameter>,
}

/// Representation of a score field which can be used in a search.
#[derive(Debug, Serialize, Deserialize, JsonSchema, FromFormField)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub enum ScoreSearchParameter {
    Title,
    Genre,
    SubTitle,
    Arranger,
    Composer,
    Annotation,
    Alias,
    Publisher,
}

impl SchemaExample for Score {
    fn example() -> Self {
        Self {
            couch_id: None,
            couch_revision: None,
            id: Some(42),
            legacy_id: None,
            title: "baum".to_string(),
            genres: vec![],
            composers: vec![],
            arrangers: vec![],
            publisher: Some("Hansl Verlag".to_string()),
            publisher_id: None,
            grade: None,
            grade_id: None,
            alias: vec!["strauch".to_string(), "teller".to_string()],
            subtitles: vec![],
            annotation: None,
            back_of_id: None,
            back_of: None,
            location: None,
            legacy_location: None,
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

impl SchemaExample for ScoreSearchParameter {
    fn example() -> Self {
        Self::Title
    }
}

impl ScoreSearchParameter {
    fn to_database_field(&self) -> String {
        match self {
            ScoreSearchParameter::Title => "title",
            ScoreSearchParameter::SubTitle => "subtitle",
            ScoreSearchParameter::Alias => "alias",
            ScoreSearchParameter::Publisher => "publisher",
            ScoreSearchParameter::Annotation => "annotation",
            ScoreSearchParameter::Composer => "composer",
            ScoreSearchParameter::Arranger => "arranger",
            ScoreSearchParameter::Genre => "genre",
        }
        .to_string()
    }
}
