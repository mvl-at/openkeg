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

use reqwest::Client;
use rocket::State;
use rocket_okapi::openapi;

use crate::archive::model::Score;
use crate::database::client::FindResponse;
use crate::openapi::ApiResult;
use crate::user::executives::{Archive, ExecutiveRole};
use crate::Config;

/// Fetch all scores which are part of the given `book`.
/// The scores are sorted as usual in books which means the following order:
///
/// . `prefix` (`None` last)
/// . `number`
/// . `suffix` (`None` last)
///
/// # Arguments
///
/// * `name`: the name of the book to fetch
/// * `conf`: the application configuration
/// * `_archive_role`: the archive role guard
/// * `client`: the client to send the database requests with
///
/// returns: Result<Json<FindResponse<Score>>, Error>
#[openapi(tag = "Archive")]
#[get("/<name>/content")]
pub async fn get_book_content(
    name: String,
    conf: &State<Config>,
    _archive_role: ExecutiveRole<Archive>,
    client: &State<Client>,
) -> ApiResult<FindResponse<Score>> {
    crate::database::score::get_book_content(conf, client, name).await
}
