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

use ldap3::LdapConnAsync;

use crate::config::Config;
use crate::members::model::Member;
use crate::members::state::Repository;
use crate::MemberStateMutex;

/// Authenticate a member against the directory server using bind.
///
/// # Arguments
///
/// * `config`: the application configuration
/// * `member_state`: the state which holds the members
/// * `username`: the username to use for authentication. this is _not_ the dn but the value of the username attributes of the member
/// * `password`: the password to use for the authentication
///
/// returns: Result<Member, ()>
///
/// # Examples
///
/// ```
/// let result = authenticate(&config, &member_state, &"willi".to_string(), &"some-secret".to_string());
/// if result.is_ok() {
///     //authentication was successful
/// } else {
///     //authentication failed
/// }
/// ```
pub async fn authenticate(
    config: &Config,
    member_state: &mut MemberStateMutex,
    username: &String,
    password: &String,
) -> Result<Member, ()> {
    debug!("try to authenticate {}", username);
    let member_state_lock = member_state.read().await;
    let member_option = member_state_lock.all_members.find(username);
    if member_option.is_none() {
        info!(
            "someone tried to authenticate with non-existing username: {}",
            username
        );
        return Err(());
    }
    let member = member_option.unwrap();
    let dn = &member.full_username;
    let ldap_config = &config.ldap;
    info!("bind to auth server: {}", ldap_config.server);
    let ldap_result = LdapConnAsync::new(&*ldap_config.server).await;
    if ldap_result.is_err() {
        error!(
            "failed to open auth session: {:#?}",
            ldap_result.err().unwrap()
        );
        return Err(());
    }
    let (conn, mut ldap) = ldap_result.unwrap();
    ldap3::drive!(conn);
    let result = ldap.simple_bind(dn, password).await;
    if result.is_err() {
        return Err(());
    }
    let res = result.unwrap();
    if res.success().is_ok() {
        info!("authenticated {}", member.username);
        return Ok(member.clone());
    }
    Err(())
}
