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

use ldap3::LdapError;
use rocket::tokio;

use crate::config::{Config, LdapConfig};
use crate::ldap::{search_entries, LdapDeserializable};
use crate::member::model::{Group, Member};
use crate::member::state::{MemberState, RegisterEntry};
use crate::MemberStateMutex;

/// Synchronize all member and groups with the directory server.
/// This includes transformations into the desired data structures which also includes sorting.
/// Note that this modifies the provided structures but they only will be modified on success.
/// If one of the fetching operations from the directory server fails, nothing will be modified in order to avoid inconsistency.
/// # Arguments
///
/// * `conf` : the application configuration
/// * `member_state` the mutex of the current member state which should be altered
pub async fn synchronize_members_and_groups(conf: &Config, member_state: &mut MemberStateMutex) {
    let ldap_conf = &conf.ldap;
    let result = fetch_results(conf, ldap_conf).await;
    if let Err(err) = result {
        warn!(
            "Unable to fetch partial data from the directory server, stop synchronizing: {:?}",
            err
        );
        return;
    }
    let (
        mut members_vector,
        mut sutlers_vector,
        mut honorary_vector,
        mut registers_vector,
        mut executives_vector,
    ) = result.expect("member vectors - checked above");

    info!("Done fetching, begin with transformation");
    let mut member_state_lock = member_state.write().await;
    fill_primitive_collections(
        conf,
        &mut member_state_lock,
        &mut members_vector,
        &mut sutlers_vector,
        &mut honorary_vector,
        &mut registers_vector,
        &mut executives_vector,
    );
    debug!("Done with copying data, begin with sorting");
    construct_members_by_register(&mut member_state_lock, members_vector, registers_vector);
    info!("Done with user synchronization")
}

/// Constructs the sorted member by register collection and saves it to the application state.
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
    conf: &Config,
    member_state: &mut MemberState,
    member_vector: &mut Vec<Member>,
    sutler_vector: &mut Vec<Member>,
    honorary_vector: &mut Vec<Member>,
    registers_vector: &mut Vec<Group>,
    executives_vector: &mut Vec<Group>,
) {
    member_state.all_members.clear();
    member_vector.sort();
    *member_vector = sort_titles_attributes(conf, member_vector);
    member_state
        .all_members
        .extend(member_vector.iter().cloned());
    member_state.sutlers.clear();
    sutler_vector.sort();
    member_state
        .sutlers
        .extend(sort_titles_attributes(conf, sutler_vector));
    member_state.honorary_members.clear();
    honorary_vector.sort();
    member_state
        .honorary_members
        .extend(sort_titles_attributes(conf, honorary_vector));
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
) -> Result<
    (
        Vec<Member>,
        Vec<Member>,
        Vec<Member>,
        Vec<Group>,
        Vec<Group>,
    ),
    LdapError,
> {
    let members = fetch_entries::<Member, Member>(
        "member",
        &ldap_conf.member_base,
        &ldap_conf.member_filter,
        conf,
    )
    .await?;
    let sutlers = fetch_entries::<Member, Member>(
        "sutlers",
        &ldap_conf.sutler_base,
        &ldap_conf.sutler_filter,
        conf,
    )
    .await?;
    let honoraries = fetch_entries::<Member, Member>(
        "honorary member",
        &ldap_conf.honorary_base,
        &ldap_conf.honorary_filter,
        conf,
    )
    .await?;
    let registers = fetch_entries::<Group, Group>(
        "registers",
        &ldap_conf.register_base,
        &ldap_conf.register_filter,
        conf,
    )
    .await?;
    let executives = fetch_entries::<Group, Group>(
        "executive roles",
        &ldap_conf.executives_base,
        &ldap_conf.executives_filter,
        conf,
    )
    .await?;
    Ok((members, sutlers, honoraries, registers, executives))
}

/// Fetch all entries of the given type and print messages.
///
/// # Arguments
///
/// * `typ` : the type of the entries which is used for messages
/// * `base` : the base dn to search in
/// * `filter` : the auth filter to use during search
/// * `conf` : the application configuration
async fn fetch_entries<R, E>(
    typ: &str,
    base: &str,
    filter: &str,
    conf: &Config,
) -> Result<Vec<R>, LdapError>
where
    E: LdapDeserializable<R>,
{
    let ldap_entries = search_entries::<R, E>(base, filter, conf).await?;
    info!(
        "Successfully received {} {} entries",
        ldap_entries.len(),
        typ
    );
    Ok(ldap_entries)
}

/// Runs the task to synchronize all member and groups and attaches it to the member state.
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

/// Sorts the titles attributes of members based on the configuration specified in `conf`.
///
/// # Arguments
///
/// * `conf` - A [Config] struct that contains the configuration information for sorting titles.
/// * `members` - A vector of [Member] structs that represent the members to sort.
///
/// # Returns
///
/// The function returns a new vector of [Member] structs with the titles attributes sorted according to the configuration.
///
/// # Examples
///
/// ```
/// let conf = Config::new();
/// let members = vec![Member::new("John", vec!["Title3", "Title1", "Title2"])];
/// let result = sort_titles_attributes(&conf, &members);
/// assert_eq!(result[0].titles, vec!["Title1", "Title2", "Title3"]);
/// ```
fn sort_titles_attributes(conf: &Config, members: &Vec<Member>) -> Vec<Member> {
    members
        .iter()
        .map(|m| {
            let mut titles = m.titles.clone();
            sort_titles_vector(conf, &mut titles);
            Member {
                titles,
                ..m.clone()
            }
        })
        .collect()
}

/// Sorts a vector of titles based on the title ordering configuration specified in `conf` inplace.
///
/// # Arguments
///
/// * `conf` - A `Config` struct that contains the configuration information for sorting titles.
/// * `titles` - A mutable reference to a vector of strings representing the titles to sort.
///
/// # Examples
///
/// ```
/// let conf = Config::new();
/// let mut titles = vec!["Title3", "Title1", "Title2"];
/// sort_titles_vector(&conf, &mut titles);
/// assert_eq!(titles, vec!["Title1", "Title2", "Title3"]);
/// ```
fn sort_titles_vector(conf: &Config, titles: &mut Vec<String>) {
    titles.sort_by_cached_key(|t| {
        conf.ldap
            .title_ordering
            .iter()
            .position(|e| e == t)
            .unwrap_or(0)
    });
}
