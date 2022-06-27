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
    pub sutler_base: String,
    pub sutler_filter: String,
    pub honorary_base: String,
    pub honorary_filter: String,
    pub register_base: String,
    pub register_filter: String,
    pub executives_base: String,
    pub executives_filter: String,
    pub member_mapping: MemberMapping,
    pub address_mapping: AddressMapping,
    pub group_mapping: GroupMapping,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            server: "ldap://localhost:389".to_string(),
            dn: None,
            password: None,
            member_base: "".to_string(),
            member_filter: "(objectClass=*)".to_string(),
            sutler_base: "".to_string(),
            sutler_filter: "(objectClass=*)".to_string(),
            honorary_base: "".to_string(),
            honorary_filter: "(objectClass=*)".to_string(),
            register_base: "".to_string(),
            register_filter: "(objectClass=*)".to_string(),
            executives_base: "".to_string(),
            executives_filter: "(objectClass=*)".to_string(),
            member_mapping: MemberMapping::default(),
            address_mapping: AddressMapping::default(),
            group_mapping: GroupMapping::default(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressMapping {
    pub street: String,
    pub house_number: String,
    pub postal_code: String,
    pub city: String,
    pub state: String,
    pub country_code: String,
}

impl Default for AddressMapping {
    fn default() -> Self {
        Self {
            street: "street".to_string(),
            house_number: "houseIdentifier".to_string(),
            postal_code: "postalCode".to_string(),
            city: "l".to_string(),
            state: "st".to_string(),
            country_code: "c".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMapping {
    pub name: String,
    pub name_plural: String,
    pub description: String,
    pub members: String,
}

impl Default for GroupMapping {
    fn default() -> Self {
        Self {
            name: "cn".to_string(),
            name_plural: "cns".to_string(),
            description: "description".to_string(),
            members: "member".to_string(),
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
