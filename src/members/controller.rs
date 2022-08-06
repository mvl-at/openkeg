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

use ldap3::tokio::task;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::api_result::{Error, Result};
use crate::config::Config;
use crate::ldap::sync::synchronize_members_and_groups;
use crate::members::model::{Crew, Member, WebMember, WebRegister};
use crate::members::photo::Photo;
use crate::members::state::Repository;
use crate::MemberStateMutex;

/// Get all members without any sensitive data.
/// Intended for the web representation of all members
#[openapi(tag = "Members")]
#[get("/")]
pub async fn all_members(member_state: &State<MemberStateMutex>) -> Result<Crew> {
    let members = member_state.read().await;
    let member_mapper: &dyn Fn(&Member) -> WebMember = &|m| WebMember::from_member(m, true);
    Ok(Json(Crew::new(
        &members.members_by_register,
        &members.sutlers,
        &members.honorary_members,
        member_mapper,
        &|r| WebRegister::from_register(r, member_mapper),
    )))
}

/// Return the profile photo of a member in the JPEG format.
///
/// # Arguments
///
/// * `username`: the username of the member whose photo is requested
/// * `member_state`: the state of all members
///
/// returns: Result<Photo, Error>
#[openapi(tag = "Members")]
#[get("/<username>/photo")]
pub async fn photo(
    username: String,
    member_state: &State<MemberStateMutex>,
) -> std::result::Result<Photo, Error> {
    let member_state_lock = member_state.read().await;
    let member_option = member_state_lock.all_members.find(&username);
    if member_option.is_none() {
        debug!("unable to find member with username {}", username);
        return Err(Error {
            err: "Not Found".to_string(),
            msg: Some("No member with such username".to_string()),
            http_status_code: Status::NotFound.code,
        });
    }
    let member = member_option.unwrap();
    Ok(Photo(member.photo.to_vec()))
}

/// Synchronize all members.
///
/// # Arguments
///
/// * `sync` - a bool which indicates if the synchronization should block this call or not
#[openapi(tag = "Members")]
#[post("/synchronize")]
pub fn synchronize(config: &State<Config>, member_state: &State<MemberStateMutex>) -> Result<()> {
    let conf_copy = config.inner().clone();
    let mut member_state_clone = member_state.inner().clone();
    let fetch_task = async move {
        synchronize_members_and_groups(&conf_copy, &mut member_state_clone).await;
    };
    task::spawn(fetch_task);
    Ok(Json(()))
}

/// Print all members to the debug console.
/// Only for debug purposes.
#[cfg(feature = "debug")]
#[openapi(tag = "Members")]
#[get("/debug-list")]
pub async fn list(member_state: &State<MemberStateMutex>) {
    debug!("{:?}", member_state.read().await.all_members);
}
