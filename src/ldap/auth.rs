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

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use ldap3::LdapConnAsync;

use crate::config::Config;
use crate::members::model::Member;
use crate::members::state::Repository;
use crate::MemberStateMutex;

/// Error which during the authentication.
#[derive(Debug)]
pub enum AuthenticationError<'u> {
    /// No such user exists.
    NonExistingUsername(&'u str),
    /// Something went wrong during opening the ldap session.
    Session,
    /// Something went wrong during the bind method.
    Bind(&'u str),
    /// The credentials are invalid.
    Credentials(&'u str),
}

impl Display for AuthenticationError<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let arguments = match self {
            AuthenticationError::NonExistingUsername(username) => {
                format!("no such username exists: '{}'", username)
            }
            AuthenticationError::Session => "failed to open the ldap session".to_string(),
            AuthenticationError::Bind(username) => format!(
                "user with username '{}' failed to bind to the ldap server",
                username
            ),
            AuthenticationError::Credentials(username) => {
                format!("invalid credentials for user with username '{}'", username)
            }
        };
        formatter.write_str(arguments.as_str())
    }
}

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
pub async fn authenticate<'a>(
    config: &Config,
    member_state: &mut MemberStateMutex,
    username: &'a str,
    password: &str,
) -> Result<Member, AuthenticationError<'a>> {
    debug!("Try to authenticate {}", username);
    let member_state_lock = member_state.read().await;
    let member = member_state_lock
        .all_members
        .find(&username.to_string())
        .ok_or(AuthenticationError::NonExistingUsername(username))?;
    let dn = &member.full_username;
    let ldap_config = &config.ldap;
    info!("Bind to auth server: {}", ldap_config.server);
    let (conn, mut ldap) = LdapConnAsync::new(&*ldap_config.server)
        .await
        .map_err(|e| {
            error!("Failed to open the auth session: {:#?}", e);
            AuthenticationError::Session
        })?;
    ldap3::drive!(conn);
    let ldap_result = ldap.simple_bind(dn, password).await.map_err(|e| {
        warn!("Failed to bind to the ldap server: {:#?}", e);
        AuthenticationError::Bind(username)
    })?;
    ldap_result
        .success()
        .map_err(|_| AuthenticationError::Credentials(username))?;
    Ok(member.clone())
}
