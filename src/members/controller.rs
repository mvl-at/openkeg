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
use ldap3::tokio::task;
use rocket::serde::json::Json;
use rocket::tokio::sync::RwLock;
use rocket::State;
use rocket_okapi::openapi;
use std::sync::Arc;

use crate::errors::Result;
use crate::ldap::synchronize_members_and_groups;
use crate::members::model::{Crew, Member, WebMember, WebRegister};
use crate::MemberState;

/// Get all members without any sensitive data.
/// Intended for the web representation of all members
#[openapi(tag = "Members")]
#[get("/")]
pub async fn all_members(member_state: &State<Arc<RwLock<MemberState>>>) -> Result<Crew> {
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

/// Synchronize all members.
///
/// # Arguments
///
/// * `sync` - a bool which indicates if the synchronization should block this call or not
#[openapi(tag = "Members")]
#[post("/synchronize")]
pub fn synchronize(
    config: &State<Config>,
    member_state: &State<Arc<RwLock<MemberState>>>,
) -> Result<()> {
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
#[openapi(tag = "Members")]
#[get("/debug-list")]
pub async fn list(member_state: &State<Arc<RwLock<MemberState>>>) {
    debug!("{:?}", member_state.read().await.all_members);
}
