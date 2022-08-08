use ldap3::{Ldap, LdapConnAsync, LdapError, Scope, SearchEntry};

use crate::Config;

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
pub mod auth;
pub mod sync;

/// A trait which ensures the deserialization capability of a struct.
pub trait LdapDeserializable<T> {
    /// Construct the struct out of a search entry
    ///
    /// # Arguments
    ///
    /// * `entry` : the entry which contains the data for constructing the struct
    /// * `config` : the configuration of the application - might be used for correct struct mappings
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> T;
}

/// Search for entries in the auth directory and construct the entities.
///
/// # Arguments
///
/// * `base` : the base dn to search for
/// * `filter` : the auth filter used for the search
/// * `config` : the application configuration
///
pub async fn search_entries<R, E>(
    base: &String,
    filter: &String,
    config: &Config,
) -> Result<Vec<R>, LdapError>
where
    E: LdapDeserializable<R>,
{
    info!(
        "Searching for in the auth server at '{}' with filter '{}'",
        base, filter
    );
    let mut ldap = open_session(config).await?;
    let (entries, _search_result) = ldap
        .search(base, Scope::Subtree, filter, vec!["*"])
        .await?
        .success()?;
    debug!(
        "Received a result, looping through {} entries",
        entries.len()
    );
    let mapped_entries = entries
        .iter()
        .map(|result_entry| {
            let entry = SearchEntry::construct(result_entry.to_owned());
            E::from_search_entry(&entry, config)
        })
        .collect();
    ldap.unbind().await?;
    Ok(mapped_entries)
}

/// Open the ldap session
///
/// # Arguments
///
/// * `config` : the application configuration used for retrieving the ldap server credentials
async fn open_session(config: &Config) -> Result<Ldap, LdapError> {
    let ldap_config = &config.ldap;
    info!("Bind to ldap server: {}", ldap_config.server);
    let (conn, mut ldap) = LdapConnAsync::new(&*ldap_config.server).await?;
    ldap3::drive!(conn);
    if ldap_config.dn.is_none() {
        warn!("Using ldap without user, this is not recommended");
    } else {
        let user = ldap_config.dn.as_ref().unwrap();
        info!("Bind ldap user with dn '{}'", user);
        let result = ldap
            .simple_bind(
                &*user,
                &*ldap_config.password.as_ref().unwrap_or(&"".to_string()),
            )
            .await?;
        result.non_error()?;
    }
    Ok(ldap)
}
