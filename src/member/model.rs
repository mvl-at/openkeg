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

use crate::config::Config;
use crate::ldap::LdapDeserializable;
use crate::member::state::{HonoraryMembers, MembersByRegister, RegisterEntry, Sutlers};
use crate::openapi::SchemaExample;
use ldap3::SearchEntry;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::Ordering;
use std::collections::{HashMap, LinkedList};
use std::hash::Hash;

/// Representation of the whole crew intended to use for the REST API.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct Crew {
    /// The musicians of the crew
    pub musicians: LinkedList<WebRegister>,
    /// The sutlers of the crew
    pub sutlers: LinkedList<WebMember>,
    /// The honorary member
    pub honorary_members: LinkedList<WebMember>,
}

/// Representation of a register intended to use for the REST API.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Register", crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct WebRegister {
    /// The name of this register
    pub name: String,
    /// The plural name of this register
    pub name_plural: String,
    /// The member which are part of this register
    pub members: LinkedList<WebMember>,
}

/// Representation of a member intended to use for the REST API.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Member", crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct WebMember {
    /// The first name of this member
    pub first_name: String,
    /// The last name of this member
    pub last_name: String,
    /// The year this member joined
    pub joining: u32,
    /// The gender of this member
    pub gender: char,
    /// Whether this member is official or not
    pub official: bool,
    /// Whether this member is active or not
    pub active: bool,
    /// The username of the user to e.g. fetch the photo
    pub username: String,
    /// The titles of the user
    pub titles: Vec<String>,
    /// Sensitive data of this member such as address and phone numbers
    /// This is only intended for authenticated users
    pub sensitives: Option<WebMemberSensitives>,
}

/// Sensitive data of a `WebMember` which is intended to be seen only by authenticated users.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    rename = "MemberSensitives",
    crate = "rocket::serde",
    rename_all = "camelCase"
)]
#[schemars(example = "Self::example")]
pub struct WebMemberSensitives {
    /// The full qualified username of the member
    pub full_username: String,
    /// The common name of the member
    pub common_name: String,
    /// The telephone numbers of the member
    pub mobile: Vec<String>,
    /// Whether this member uses whatsapp or not
    pub whatsapp: bool,
    /// The birthday of the member
    pub birthday: String,
    /// The mail addresses of the member
    pub mail: Vec<String>,
    /// The actual address oft the member
    pub address: Option<WebAddress>,
}

/// The address of a member intended for web usage.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Address", crate = "rocket::serde", rename_all = "camelCase")]
#[schemars(example = "Self::example")]
pub struct WebAddress {
    /// The street of this address
    pub street: String,
    /// The house number of this address
    pub house_number: String,
    /// The postal code of this address
    pub postal_code: String,
    /// The city of this address
    pub city: String,
    /// The state of this address
    pub state: String,
    /// The country code of this address
    pub country_code: String,
}

impl SchemaExample for Crew {
    fn example() -> Self {
        Self {
            musicians: LinkedList::from([WebRegister::example()]),
            sutlers: LinkedList::from([WebMember::example()]),
            honorary_members: LinkedList::from([WebMember::example()]),
        }
    }
}

impl Crew {
    pub fn new(
        musicians: &MembersByRegister,
        sutlers: &Sutlers,
        honorary_members: &HonoraryMembers,
        member_mapper: &dyn Fn(&Member) -> WebMember,
        register_mapper: &dyn Fn(&RegisterEntry) -> WebRegister,
    ) -> Self {
        Self {
            musicians: musicians.iter().map(register_mapper).collect(),
            sutlers: sutlers.iter().map(member_mapper).collect(),
            honorary_members: honorary_members.iter().map(member_mapper).collect(),
        }
    }
}

impl SchemaExample for WebRegister {
    fn example() -> Self {
        Self {
            name: "Kukuruz".to_string(),
            name_plural: "Kukuruzn".to_string(),
            members: LinkedList::from([WebMember::example()]),
        }
    }
}

impl WebRegister {
    pub fn from_register(
        register: &RegisterEntry,
        member_mapper: &dyn Fn(&Member) -> WebMember,
    ) -> Self {
        Self {
            name: register.register.name.to_string(),
            name_plural: register.register.name_plural.to_string(),
            members: register.members.iter().map(member_mapper).collect(),
        }
    }
}

impl SchemaExample for WebMember {
    fn example() -> Self {
        Self {
            first_name: "Karl".to_string(),
            last_name: "Steinscheisser".to_string(),
            joining: 2003,
            gender: 'm',
            official: true,
            active: true,
            username: "karli".to_string(),
            titles: vec!["Held".to_string()],
            sensitives: Some(WebMemberSensitives::example()),
        }
    }
}

impl WebMember {
    /// Create a `WebMember` from a member.
    ///
    /// # Arguments
    ///
    /// * `member` : the `Member` to map
    /// * `sensitive` : whether to also map sensitive data or not
    pub fn from_member(member: &Member, sensitive: bool) -> Self {
        Self {
            first_name: member.first_name.to_string(),
            last_name: member.last_name.to_string(),
            joining: member.joining,
            gender: member.gender,
            official: member.official,
            active: member.active,
            username: member.username.to_string(),
            titles: member.titles.clone(),
            sensitives: sensitive.then(|| WebMemberSensitives::from_member(member)),
        }
    }
}

impl SchemaExample for WebMemberSensitives {
    fn example() -> Self {
        Self {
            full_username: "uid=karl".to_string(),
            common_name: "uid=karl".to_string(),
            mobile: vec![
                "+43 664 91828374".to_string(),
                "+43 699 28184853".to_string(),
            ],
            whatsapp: false,
            birthday: "1996-05-06".to_string(),
            mail: vec![
                "joker@batman.org".to_string(),
                "kar@steinscheisser.at".to_string(),
            ],
            address: Some(WebAddress::example()),
        }
    }
}

impl WebMemberSensitives {
    pub fn from_member(member: &Member) -> Self {
        Self {
            full_username: member.full_username.clone(),
            common_name: member.common_name.clone(),
            mobile: member.mobile.clone(),
            whatsapp: member.whatsapp,
            birthday: member.birthday.to_string(),
            mail: member.mail.clone(),
            address: member.address.as_ref().map(WebAddress::from_address),
        }
    }
}

impl SchemaExample for WebAddress {
    fn example() -> Self {
        Self {
            street: "Kempfendorf".to_string(),
            house_number: "2".to_string(),
            postal_code: "2285".to_string(),
            city: "Leopoldsdorf i.M.".to_string(),
            state: "Niederösterreich".to_string(),
            country_code: "AT".to_string(),
        }
    }
}

impl WebAddress {
    pub fn from_address(address: &Address) -> Self {
        Self {
            street: address.street.to_string(),
            house_number: address.house_number.to_string(),
            postal_code: address.postal_code.to_string(),
            city: address.city.to_string(),
            state: address.state.to_string(),
            country_code: address.country_code.to_string(),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
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
    pub titles: Vec<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Address {
    pub street: String,
    pub house_number: String,
    pub postal_code: String,
    pub city: String,
    pub state: String,
    pub country_code: String,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Group {
    pub name: String,
    pub name_plural: String,
    pub description: String,
    pub members: Vec<String>,
}

impl PartialOrd<Self> for Member {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Member {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.joining < other.joining {
            return Ordering::Less;
        }
        if self.joining > other.joining {
            return Ordering::Greater;
        }
        let lastname = self.last_name.cmp(&other.last_name);
        if lastname != Ordering::Equal {
            return lastname;
        }
        let firstname = self.first_name.cmp(&other.first_name);
        if firstname != Ordering::Equal {
            return firstname;
        }
        Ordering::Equal
    }
}

impl LdapDeserializable<Member> for Member {
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> Member {
        let attrs = &entry.attrs;
        let mapping = &config.ldap.member_mapping;
        Member {
            username: string_or_blank(&mapping.username, attrs)[0].to_string(),
            full_username: entry.dn.to_string(),
            first_name: string_or_blank(&mapping.first_name, attrs)[0].to_string(),
            last_name: string_or_blank(&mapping.last_name, attrs)[0].to_string(),
            common_name: string_or_blank(&mapping.common_name, attrs)[0].to_string(),
            whatsapp: bool_or_false(&mapping.whatsapp, attrs),
            joining: string_or_blank(&mapping.joining, attrs)[0]
                .parse::<u32>()
                .unwrap_or(0),
            listed: bool_or_false(&mapping.listed, attrs),
            official: bool_or_false(&mapping.official, attrs),
            gender: string_or_blank(&mapping.gender, attrs)[0]
                .chars()
                .next()
                .unwrap_or('u'),
            active: bool_or_false(&mapping.active, attrs),
            mobile: string_or_empty(&mapping.mobile, attrs),
            birthday: string_or_blank(&mapping.birthday, attrs)[0].to_string(),
            mail: string_or_empty(&mapping.mail, attrs),
            photo: entry
                .bin_attrs
                .get(&mapping.photo)
                .unwrap_or(&vec![])
                .iter()
                .next()
                .unwrap_or(&vec![])
                .to_owned(),
            titles: string_or_empty(&mapping.titles, attrs),
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
            &[
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
            street: string_or_blank(&mapping.street, attrs)[0].to_string(),
            house_number: string_or_blank(&mapping.house_number, attrs)[0].to_string(),
            postal_code: string_or_blank(&mapping.postal_code, attrs)[0].to_string(),
            city: string_or_blank(&mapping.city, attrs)[0].to_string(),
            state: string_or_blank(&mapping.state, attrs)[0].to_string(),
            country_code: string_or_blank(&mapping.country_code, attrs)[0].to_string(),
        })
    }
}

impl LdapDeserializable<Group> for Group {
    fn from_search_entry(entry: &SearchEntry, config: &Config) -> Group {
        let attrs = &entry.attrs;
        let mapping = &config.ldap.group_mapping;
        Group {
            name: string_or_blank(&mapping.name, attrs)[0].to_string(),
            name_plural: string_or_blank(&mapping.name_plural, attrs)[0].to_string(),
            description: string_or_blank(&mapping.description, attrs)[0].to_string(),
            members: attrs
                .get(mapping.members.as_str())
                .unwrap_or(&vec![])
                .clone(),
        }
    }
}

impl PartialOrd<Self> for Group {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Group {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

/// Extract either the strings out of a vector map or fill the vector with one empty string if the attribute does not exist.
///
/// # Arguments
///
/// * `attribute` : the attribute whose value should be extracted from the map
/// * `attrs` : the map of the attributes with the corresponding values
fn string_or_blank(attribute: &str, attrs: &HashMap<String, Vec<String>>) -> Vec<String> {
    attrs
        .get(attribute)
        .unwrap_or(&vec!["".to_string()])
        .clone()
}

/// Extract either the strings out of a vector map or return an empty one if the attribute does not exist.
///
/// # Arguments
///
/// * `attribute` : the attribute whose value should be extracted from the map
/// * `attrs` : the map of the attributes with the corresponding values
fn string_or_empty(attribute: &str, attrs: &HashMap<String, Vec<String>>) -> Vec<String> {
    attrs.get(attribute).unwrap_or(&vec![]).clone()
}

/// Extract the first value of the attribute map or return `false` if none exist
///
/// # Arguments
///
/// * `attribute` : the attribute whose value should be extracted from the map
/// * `attrs` : the map of the attributes with the corresponding values
fn bool_or_false(attribute: &str, attrs: &HashMap<String, Vec<String>>) -> bool {
    attrs.get(attribute).unwrap_or(&vec!["".to_string()])[0].eq_ignore_ascii_case("true")
}

/// Returns `true` if the map contains all the keys and `false` if at least one does not.
///
/// # Arguments
///
/// * `map` : the map which should contain all the keys
/// * `keys` : a vector of all the required keys
fn contains_all<K, V>(map: &HashMap<K, V>, keys: &[K]) -> bool
where
    K: Hash + Eq,
{
    keys.iter()
        .map(|k| map.contains_key(k))
        .reduce(|k, l| k && l)
        .unwrap_or(false)
}
