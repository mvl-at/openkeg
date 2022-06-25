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

use crate::config::Config;
use futures::FutureExt;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::errors::Result;
use crate::ldap;

/// Synchronize all members.
///
/// # Arguments
///
/// * `sync` - a bool which indicates if the synchronization should block this call or not
#[openapi(tag = "Members")]
#[post("/synchronize?<sync>")]
pub fn synchronize(sync: bool, config: &State<Config>) -> Result<()> {
    ldap::members(config.inner()).now_or_never();
    Ok(Json(()))
}
