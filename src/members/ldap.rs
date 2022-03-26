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

use ldap3::{LdapConnAsync, LdapConnSettings};

use crate::config::Config;

pub async fn open_session(config: Config) {
    let ldap_config = config.ldap;
    eprintln!("open session to ldap server: {}", ldap_config.server);
    let ldap_result = LdapConnAsync::new(&*ldap_config.server).await;
    if ldap_result.is_err() {
        eprintln!(
            "failed to open ldap session: {:#?}",
            ldap_result.err().unwrap()
        );
        return;
    }
    let (conn, mut ldap) = ldap_result.unwrap();
    ldap3::drive!(conn);
    if ldap_config.dn.is_none() {
        eprintln!("using ldap without user");
    } else {
        let user = ldap_config.dn.unwrap();
        eprintln!("bind ldap user with dn '{}'", user);
        let result = ldap
            .simple_bind(&*user, &*ldap_config.password.unwrap_or("".to_string()))
            .await;
        if result.is_err() {
            eprintln!("failed to bind user: {:#?}", result.err().unwrap())
        } else {
            eprintln!("bind result: {}", result.unwrap());
        }
    }
}
