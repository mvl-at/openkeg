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
use crate::ldap::LdapDeserializable;
use ldap3::SearchEntry;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Member {
    pub username: String,
    pub full_username: String,
    pub first_name: String,
    pub last_name: String,
    pub common_name: String,
    pub whatsapp: bool,
    pub joining: u32,
    pub listed: bool,
    pub official: bool,
    pub gender: char,
    pub active: bool,
    pub mobile: Vec<String>,
    pub birthday: String,
    pub mail: Vec<String>,
    pub photo: Vec<u8>,
    pub address: Option<Address>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Address {
    pub street: String,
    pub house_number: String,
    pub postal_code: String,
    pub city: String,
    pub state: String,
    pub country_code: String,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Group {
    pub name: String,
    pub name_plural: String,
    pub description: String,
    pub members: Vec<String>,
}

impl LdapDeserializable<Member> for Member {
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> Member {
        let attrs = &entry.attrs;
        let mapping = &config.ldap.member_mapping;
        Member {
            username: string_or_empty(&mapping.username, attrs)[0].to_string(),
            full_username: entry.dn.to_string(),
            first_name: string_or_empty(&mapping.first_name, attrs)[0].to_string(),
            last_name: string_or_empty(&mapping.last_name, attrs)[0].to_string(),
            common_name: string_or_empty(&mapping.common_name, attrs)[0].to_string(),
            whatsapp: bool_or_false(&mapping.whatsapp, attrs),
            joining: string_or_empty(&mapping.joining, attrs)[0]
                .parse::<u32>()
                .unwrap_or(0),
            listed: bool_or_false(&mapping.listed, attrs),
            official: bool_or_false(&mapping.official, attrs),
            gender: string_or_empty(&mapping.gender, attrs)[0]
                .chars()
                .next()
                .unwrap_or('u'),
            active: bool_or_false(&mapping.active, attrs),
            mobile: string_or_empty(&mapping.username, attrs),
            birthday: string_or_empty(&mapping.birthday, attrs)[0].to_string(),
            mail: string_or_empty(&mapping.mail, attrs),
            photo: entry
                .bin_attrs
                .get(&mapping.photo)
                .unwrap_or(&vec![])
                .iter()
                .next()
                .unwrap_or(&vec![])
                .to_owned(),
            address: Address::from_search_entry(entry, config),
        }
    }
}

impl LdapDeserializable<Option<Address>> for Address {
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> Option<Address> {
        let attrs = &entry.attrs;
        let mapping = &config.ldap.address_mapping;
        if !contains_all(
            attrs,
            &vec![
                mapping.country_code.to_string(),
                mapping.postal_code.to_string(),
                mapping.city.to_string(),
                mapping.house_number.to_string(),
                mapping.state.to_string(),
                mapping.street.to_string(),
            ],
        ) {
            return None;
        }
        Some(Address {
            street: string_or_empty(&mapping.street, attrs)[0].to_string(),
            house_number: string_or_empty(&mapping.house_number, attrs)[0].to_string(),
            postal_code: string_or_empty(&mapping.postal_code, attrs)[0].to_string(),
            city: string_or_empty(&mapping.city, attrs)[0].to_string(),
            state: string_or_empty(&mapping.state, attrs)[0].to_string(),
            country_code: string_or_empty(&mapping.country_code, attrs)[0].to_string(),
        })
    }
}

impl LdapDeserializable<Group> for Group {
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> Group {
        let attrs = &entry.attrs;
        let mapping = &config.ldap.group_mapping;
        Group {
            name: string_or_empty(&mapping.name, attrs)[0].to_string(),
            name_plural: string_or_empty(&mapping.name_plural, attrs)[0].to_string(),
            description: string_or_empty(&mapping.description, attrs)[0].to_string(),
            members: attrs
                .get(mapping.members.as_str())
                .unwrap_or(&vec![])
                .clone(),
        }
    }
}

/// Extract either the strings out of a vector map or fill the vector with one empty string if the attribute does not exist.
///
/// # Arguments
///
/// * `attribute` : the attribute whose value should be extracted from the map
/// * `attrs` : the map of the attributes with the corresponding values
fn string_or_empty(attribute: &String, attrs: &HashMap<String, Vec<String>>) -> Vec<String> {
    attrs
        .get(attribute)
        .unwrap_or(&vec!["".to_string()])
        .clone()
}

/// Extract the first value of the attribute map or return `false` if none exist
///
/// # Arguments
///
/// * `attribute` : the attribute whose value should be extracted from the map
/// * `attrs` : the map of the attributes with the corresponding values
fn bool_or_false(attribute: &String, attrs: &HashMap<String, Vec<String>>) -> bool {
    attrs.get(attribute).unwrap_or(&vec!["".to_string()])[0].eq_ignore_ascii_case("true")
}

/// Returns `true` if the map contains all the keys and `false` if at least one does not.
///
/// # Arguments
///
/// * `map` : the map which should contain all the keys
/// * `keys` : a vector of all the required keys
fn contains_all<K, V>(map: &HashMap<K, V>, keys: &Vec<K>) -> bool
where
    K: Hash + Eq,
{
    keys.iter()
        .map(|k| map.contains_key(k))
        .reduce(|k, l| k && l)
        .unwrap_or(false)
}
