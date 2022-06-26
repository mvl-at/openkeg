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

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ldap: LdapConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ldap: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LdapConfig {
    pub server: String,
    pub dn: Option<String>,
    pub password: Option<String>,
    pub member_base: String,
    pub member_filter: String,
    pub member_mapping: MemberMapping,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            server: "ldap://localhost:389".to_string(),
            dn: None,
            password: None,
            member_base: "".to_string(),
            member_filter: "(objectClass=*)".to_string(),
            member_mapping: MemberMapping::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemberMapping {
    pub username: String,
    pub full_username: String,
    pub first_name: String,
    pub last_name: String,
    pub common_name: String,
    pub whatsapp: String,
    pub joining: String,
    pub listed: String,
    pub official: String,
    pub gender: String,
    pub active: String,
    pub mobile: String,
    pub birthday: String,
    pub mail: String,
    pub photo: String,
}

impl Default for MemberMapping {
    fn default() -> Self {
        MemberMapping {
            username: "uid".to_string(),
            full_username: "dn".to_string(),
            first_name: "givenName".to_string(),
            last_name: "sn".to_string(),
            common_name: "cn".to_string(),
            whatsapp: "wa".to_string(),
            joining: "joining".to_string(),
            listed: "listed".to_string(),
            official: "official".to_string(),
            gender: "gender".to_string(),
            active: "active".to_string(),
            mobile: "mobile".to_string(),
            birthday: "birthday".to_string(),
            mail: "mail".to_string(),
            photo: "jpegPhoto".to_string(),
        }
    }
}

pub fn read_config() -> Figment {
    Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("keg.toml").nested())
        .merge(Env::prefixed("KEG_").global())
        .select(Profile::from_env_or("KEG_PROFILE", "default"))
}
