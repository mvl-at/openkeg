// OpenKeg, the lightweight backend of the Musikverein Leopoldsdorf.
// Copyright (C) 2023  Richard Stöckl
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

use ical;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::calendar::model::{CalendarType, Event};
use crate::config::Config;
use crate::openapi::{ApiError, ApiResult};

/// Retrieves all events from a calendar based on the specified `cal_type`.
///
/// # Arguments
///
/// * `cal_type` - A [CalendarType] enum value indicating the type of calendar to retrieve events from.
/// * `conf` - The configuration information, including the URLs for the calendars.
///
/// # Returns
///
/// If the events are retrieved successfully, the function returns a [Vec<Event>] wrapped in an [ApiResult].
///
/// If an error occurs during the retrieval process, the function returns an [ApiError] with an appropriate error message.
///
/// # Examples
///
/// ```
/// let cal_type = CalendarType::Public;
/// let conf = State::new(Config::new());
/// let result = get_all_events(cal_type, &conf);
/// assert!(result.is_ok());
/// ```
#[openapi(tag = "Calendar")]
#[get("/?<cal_type>")]
pub async fn get_all_events(cal_type: CalendarType, conf: &State<Config>) -> ApiResult<Vec<Event>> {
    let calendar_config = &conf.calendar;
    let url = match cal_type {
        CalendarType::Public => &calendar_config.ical_url,
        CalendarType::Internal => &calendar_config.ical_internal_url,
    };
    let ical_body_future = reqwest::get(url).await.map_err(|e| {
        log::error!("Unable to retrieve the calendar from the ical url {}", e);
        upstream_error()
    })?;
    let ical_body = ical_body_future.bytes().await.map_err(|e| {
        log::error!("Unable to read the body from the calendar response {}", e);
        upstream_error()
    })?;
    let parser = ical::IcalParser::new(Cursor::new(ical_body));
    let mut parse_result = Ok(());
    let events: Vec<Event> = parser
        .flat_map(|c| {
            c.map_err(|e| {
                log::error!("Unable to parse calendar {}", e);
                parse_result = Err(upstream_error())
            })
            .map(|i| i.events)
            .unwrap_or_default()
        })
        .map(|e| Event::from(&e))
        .collect();
    Ok(Json(events))
}

/// Returns an [ApiError] indicating an upstream error during calendar retrieval.
///
/// The returned error has the error message "Internal Error", the message "Unable to retrieve the calendar from upstream", and the HTTP status code set to `Status::BadGateway.code`.
///
/// # Examples
///
/// ```
/// let error = upstream_error();
/// assert_eq!(error.err, "Internal Error");
/// assert_eq!(error.msg, Some("Unable to retrieve the calendar from upstream"));
/// assert_eq!(error.http_status_code, Status::BadGateway.code);
/// ```
fn upstream_error() -> ApiError {
    ApiError {
        err: "Internal Error".to_string(),
        msg: Some("Unable to retrieve the calendar from upstream".to_string()),
        http_status_code: Status::BadGateway.code,
    }
}
