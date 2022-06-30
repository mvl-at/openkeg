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

use std::collections::{HashSet, LinkedList};
use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use crate::members::model::{Group, Member};
use crate::MemberStateMutex;

/// All members with no further order
pub type AllMembers = HashSet<Member>;
/// All registers with no further order
pub type Registers = LinkedList<Group>;
/// All executive roles with no further order
pub type Executives = HashSet<Group>;
/// All members grouped by their register.
/// Registers are ordered by their name and members are ordered by their joining, lastname and firstname
pub type MembersByRegister = LinkedList<RegisterEntry>;
/// All members which are sutlers
pub type Sutlers = LinkedList<Member>;
/// All honorary members
pub type HonoraryMembers = LinkedList<Member>;

pub trait Repository<ID, E> {
    fn find(&self, id: &ID) -> Option<&E>;
}

impl Repository<String, Member> for AllMembers {
    fn find(&self, id: &String) -> Option<&Member> {
        self.iter()
            .filter(|m| {
                m.username.eq_ignore_ascii_case(id)
                    || m.mail.iter().any(|mail| mail.eq_ignore_ascii_case(id))
            })
            .next()
    }
}

/// The state of all member data
pub struct MemberState {
    pub all_members: AllMembers,
    pub registers: Registers,
    pub executives: Executives,
    pub members_by_register: MembersByRegister,
    pub sutlers: Sutlers,
    pub honorary_members: HonoraryMembers,
}

impl MemberState {
    pub fn mutex() -> MemberStateMutex {
        Arc::new(RwLock::new(MemberState {
            all_members: AllMembers::new(),
            registers: Registers::new(),
            executives: Executives::new(),
            members_by_register: MembersByRegister::new(),
            sutlers: Sutlers::new(),
            honorary_members: HonoraryMembers::new(),
        }))
    }
}

#[derive(Clone)]
/// An entry which holds a register and all corresponding members
pub struct RegisterEntry {
    /// The register of this entry
    pub register: Group,
    /// The members of this entry
    pub members: LinkedList<Member>,
}
