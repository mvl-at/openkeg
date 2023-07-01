// OpenKeg, the lightweight backend of the Musikverein Leopoldsdorf.
// Copyright (C) 2023  Richard Stöckl
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

use std::collections::HashMap;

use ical::parser::ical::component::IcalEvent;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

use crate::openapi::SchemaExample;

/// The type of the calendar.
/// The public calendar which contains events everybody can attend.
/// An internal calendar which contains preparations, exercises and similar events.
#[derive(Serialize, Deserialize, JsonSchema, FromFormField)]
pub enum CalendarType {
    Public,
    Internal,
}

/// An event which is a simple excerpt from an ical calendar.
/// It features the properties the ical server propagates.
/// A reference which of them are utilized can be found at https://www.rfc-editor.org/rfc/rfc5545.
/// However, this structure also supports properties which are not covered by this rfc.
#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(example = "Self::example")]
pub struct Event {
    /// The map which contains all properties.
    /// Maps property names to the values.
    properties: HashMap<String, EventProperty>,
}

impl Event {
    pub fn from(ical_event: &IcalEvent) -> Self {
        let properties: HashMap<String, EventProperty> = ical_event
            .properties
            .iter()
            .map(|property| {
                (
                    property.name.to_lowercase(),
                    EventProperty {
                        value: property.value.clone(),
                        params: property.params.clone().map_or(HashMap::new(), |params| {
                            params
                                .iter()
                                .map(|param| {
                                    let values: Vec<String> = param.1.clone();
                                    (param.0.to_lowercase(), values)
                                })
                                .collect()
                        }),
                    },
                )
            })
            .collect();
        Event { properties }
    }
}

impl SchemaExample for Event {
    fn example() -> Self {
        Self {
            properties: Default::default(),
        }
    }
}

/// A single event property.
/// This structure contains the value to an ical event property.
/// In addition, it contains the parameters of the value.
#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(example = "Self::example")]
pub struct EventProperty {
    /// The value of the property.
    value: Option<String>,
    /// Maps parameter names to the parameter values.
    params: HashMap<String, Vec<String>>,
}

impl SchemaExample for EventProperty {
    fn example() -> Self {
        Self {
            value: Some("world".to_string()),
            params: Default::default(),
        }
    }
}
