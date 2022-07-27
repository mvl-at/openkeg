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

use crate::api_result::Result;
use crate::archive::database::request;
use crate::archive::model::CountStatistic;
use crate::archive::statistic::CountStatisticType;
use crate::Config;

pub async fn count_statistic(
    conf: &Config,
    client: &Client,
    subject: CountStatisticType,
) -> Result<CountStatistic> {
    let db_mapping = &conf.database.database_mapping;
    let api_url = match subject {
        CountStatisticType::Genres => &db_mapping.genres_statistic,
        CountStatisticType::Arrangers => &db_mapping.arrangers_statistic,
        CountStatisticType::Composers => &db_mapping.composers_statistic,
        CountStatisticType::Publishers => &db_mapping.publishers_statistic,
    };
    let mut parameters = HashMap::new();
    parameters.insert("group".to_string(), "true".to_string());
    parameters.insert(
        "partition".to_string(),
        conf.database.score_partition.to_string(),
    );
    request(
        conf,
        client,
        Box::new(|r| r),
        Method::GET,
        api_url,
        &parameters,
    )
    .await
    .map(|r| Json(r))
}
