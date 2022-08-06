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

use crate::Config;
use ldap3::{Ldap, LdapConnAsync, LdapError, Scope, SearchEntry};

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
    let ldap_result = open_session(config).await;
    if ldap_result.is_err() {
        error!("Failed to connect to the auth server");
        return Err(LdapError::EndOfStream);
    }
    let mut ldap = ldap_result.unwrap();
    let search_result = ldap
        .search(base, Scope::Subtree, filter, vec!["*"])
        .await?
        .success();
    debug!("Received a search result");
    if search_result.is_err() {
        let err = search_result.unwrap_err();
        error!("Retrieved auth error: {:?}", err);
        return Err(err);
    }
    let search = search_result.unwrap();
    debug!("Looping through {} results", search.0.len());
    let entries = search
        .0
        .iter()
        .map(|result_entry| {
            let entry = SearchEntry::construct(result_entry.to_owned());
            E::from_search_entry(&entry, config)
        })
        .collect();
    ldap.unbind().await?;
    Ok(entries)
}

/// Open the ldap session
///
/// # Arguments
///
/// * `config` : the application configuration used for retrieving the ldap server credentials
async fn open_session(config: &Config) -> Result<Ldap, ()> {
    let ldap_config = &config.ldap;
    info!("Bind to ldap server: {}", ldap_config.server);
    let ldap_result = LdapConnAsync::new(&*ldap_config.server).await;
    if ldap_result.is_err() {
        error!(
            "Failed to open ldap session: {:#?}",
            ldap_result.err().unwrap()
        );
        return Err(());
    }
    let (conn, mut ldap) = ldap_result.unwrap();
    ldap3::drive!(conn);
    if ldap_config.dn.is_none() {
        warn!("Using ldap without user");
    } else {
        let user = ldap_config.dn.as_ref().unwrap();
        info!("Bind ldap user with dn '{}'", user);
        let result = ldap
            .simple_bind(
                &*user,
                &*ldap_config.password.as_ref().unwrap_or(&"".to_string()),
            )
            .await;
        if result.is_err() {
            error!("Failed to bind user: {:#?}", result.err().unwrap())
        } else {
            let res = result.as_ref().unwrap();
            let error_option = res.clone().non_error().err();
            if error_option.is_some() {
                let error = error_option.unwrap();
                error!("Failed to bind({}): {} ({:?})", res.rc, res.text, error);
            }
        }
    }
    Ok(ldap)
}
