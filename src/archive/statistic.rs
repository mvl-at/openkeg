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
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::archive::model::CountStatistic;
use crate::database::statistic::count_statistic;
use crate::openapi::ApiResult as JsonResult;
use crate::Config;

/// Representation of a score field which can be used in a search.
#[derive(Serialize, Deserialize, JsonSchema, FromFormField)]
pub enum CountStatisticType {
    Genres,
    Arrangers,
    Composers,
    Publishers,
    Locations,
    Books,
}

/// Fetch the statistic for various items such as genres with their count.
#[openapi(tag = "Archive")]
#[get("/counts?<subject>")]
pub async fn get_count_statistic(
    subject: CountStatisticType,
    conf: &State<Config>,
    client: &State<Client>,
) -> JsonResult<CountStatistic> {
    count_statistic(conf, client, subject).await
}
