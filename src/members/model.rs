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
use ldap3::SearchEntry;
use std::collections::HashMap;

#[derive(Debug)]
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
}

impl Member {
    pub fn from_search_entry(entry: &SearchEntry, config: &Config) -> Member {
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
            photo: vec![],
        }
    }
}

fn string_or_empty(attribute: &String, attrs: &HashMap<String, Vec<String>>) -> Vec<String> {
    attrs
        .get(attribute)
        .unwrap_or(&vec!["".to_string()])
        .clone()
}

fn bool_or_false(attribute: &String, attrs: &HashMap<String, Vec<String>>) -> bool {
    attrs.get(attribute).unwrap_or(&vec!["".to_string()])[0].eq_ignore_ascii_case("true")
}
