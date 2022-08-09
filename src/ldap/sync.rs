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

use std::time::Duration;

use rocket::tokio;

use crate::config::{Config, LdapConfig};
use crate::ldap::{search_entries, LdapDeserializable};
use crate::members::model::{Group, Member};
use crate::members::state::{MemberState, RegisterEntry};
use crate::MemberStateMutex;

/// Synchronize all members and groups with the directory server.
/// This includes transformations into the desired data structures which also includes sorting.
/// Note that this modifies the provided structures but they only will be modified on success.
/// If one of the fetching operations from the directory server fails, nothing will be modified in order to avoid inconsistency.
/// # Arguments
///
/// * `conf` : the application configuration
/// * `member_state` the mutex of the current member state which should be altered
pub async fn synchronize_members_and_groups(conf: &Config, member_state: &mut MemberStateMutex) {
    let ldap_conf = &conf.ldap;
    let optionals = fetch_results(conf, ldap_conf).await;
    if optionals.is_none() {
        return;
    }
    let (
        mut members_vector,
        mut sutlers_vector,
        mut honorary_option,
        mut registers_vector,
        mut executives_vector,
    ) = optionals.unwrap();

    info!("Done fetching, begin with transformation");
    let mut member_state_lock = member_state.write().await;
    fill_primitive_collections(
        &mut member_state_lock,
        &mut members_vector,
        &mut sutlers_vector,
        &mut honorary_option,
        &mut registers_vector,
        &mut executives_vector,
    );
    debug!("Done with copying data, begin with sorting");

    construct_members_by_register(&mut member_state_lock, members_vector, registers_vector);
    info!("Done with user synchronization")
}

/// Constructs the sorted members by register collection and saves it to the application state.
fn construct_members_by_register(
    member_state: &mut MemberState,
    member_result: Vec<Member>,
    registers_result: Vec<Group>,
) {
    let members_by_register = &mut member_state.members_by_register;
    members_by_register.clear();
    members_by_register.extend(registers_result.iter().map(|register| {
        let register_members = member_result
            .iter()
            .filter(|m| register.members.contains(&m.full_username))
            .cloned()
            .collect();
        RegisterEntry {
            register: register.clone(),
            members: register_members,
        }
    }));
}

/// Helper function to sort and assign primitive collections.
fn fill_primitive_collections(
    member_state: &mut MemberState,
    member_vector: &mut Vec<Member>,
    sutler_vector: &mut Vec<Member>,
    honorary_vector: &mut Vec<Member>,
    registers_vector: &mut Vec<Group>,
    executives_vector: &mut Vec<Group>,
) {
    member_state.all_members.clear();
    member_vector.sort();
    member_state
        .all_members
        .extend(member_vector.iter().cloned());
    member_state.sutlers.clear();
    sutler_vector.sort();
    member_state.sutlers.extend(sutler_vector.iter().cloned());
    member_state.honorary_members.clear();
    honorary_vector.sort();
    member_state
        .honorary_members
        .extend(honorary_vector.iter().cloned());
    member_state.registers.clear();
    registers_vector.sort();
    member_state
        .registers
        .extend(registers_vector.iter().cloned());
    member_state.executives.clear();
    member_state
        .executives
        .extend(executives_vector.iter().cloned());
}

/// Helper function to fetch entries and return them all or none is at least one was not successful.
async fn fetch_results(
    conf: &Config,
    ldap_conf: &LdapConfig,
) -> Option<(
    Vec<Member>,
    Vec<Member>,
    Vec<Member>,
    Vec<Group>,
    Vec<Group>,
)> {
    let stop_str = "Unable to fetch partial data from the directory server, stop synchronizing";
    let member_option = fetch_entries::<Member, Member>(
        "members",
        &ldap_conf.member_base,
        &ldap_conf.member_filter,
        conf,
    )
    .await;
    if member_option.is_none() {
        warn!("{}", stop_str);
        return None;
    }
    let sutler_option = fetch_entries::<Member, Member>(
        "sutlers",
        &ldap_conf.sutler_base,
        &ldap_conf.sutler_filter,
        conf,
    )
    .await;
    if sutler_option.is_none() {
        warn!("{}", stop_str);
        return None;
    }
    let honorary_option = fetch_entries::<Member, Member>(
        "honorary members",
        &ldap_conf.honorary_base,
        &ldap_conf.honorary_filter,
        conf,
    )
    .await;
    if honorary_option.is_none() {
        warn!("{}", stop_str);
        return None;
    }

    let registers_option = fetch_entries::<Group, Group>(
        "registers",
        &ldap_conf.register_base,
        &ldap_conf.register_filter,
        conf,
    )
    .await;
    if registers_option.is_none() {
        warn!("{}", stop_str);
        return None;
    }

    let executives_option = fetch_entries::<Group, Group>(
        "executive roles",
        &ldap_conf.executives_base,
        &ldap_conf.executives_filter,
        conf,
    )
    .await;
    if executives_option.is_none() {
        warn!("{}", stop_str);
        return None;
    }
    Some((
        member_option.unwrap(),
        sutler_option.unwrap(),
        honorary_option.unwrap(),
        registers_option.unwrap(),
        executives_option.unwrap(),
    ))
}

/// Fetch all entries of the given type and print messages.
///
/// # Arguments
///
/// * `typ` : the type of the entries which is used for messages
/// * `base` : the base dn to search in
/// * `filter` : the auth filter to use during search
/// * `conf` : the application configuration
async fn fetch_entries<R, E>(typ: &str, base: &str, filter: &str, conf: &Config) -> Option<Vec<R>>
where
    E: LdapDeserializable<R>,
{
    let ldap_result = search_entries::<R, E>(base, filter, conf).await;
    if ldap_result.is_err() {
        warn!("Unable to fetch {} from the directory server", typ);
        return None;
    }
    let ldap_entries = ldap_result.unwrap();
    info!(
        "Successfully received {} {} entries",
        ldap_entries.len(),
        typ
    );
    Some(ldap_entries)
}

/// Runs the task to synchronize all members and groups and attaches it to the member state.
/// This task runs periodically as configured and thus will run as long as the application lives.
/// # Arguments
///
/// * `conf`: the application configuration
/// * `member_state`: the state which should be updated periodically
///
/// returns: ()
pub async fn member_synchronization_task(conf: &Config, member_state: &mut MemberStateMutex) {
    let mut interval =
        tokio::time::interval(Duration::from_secs(conf.ldap.synchronization_interval));
    loop {
        interval.tick().await;
        info!("Running scheduled user synchronization");
        synchronize_members_and_groups(conf, member_state).await;
    }
}
