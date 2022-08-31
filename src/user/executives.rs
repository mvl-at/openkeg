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

use okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::outcome::Outcome::{Forward, Success};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

use crate::config::ExecutiveMapping;
use crate::member::model::Member;
use crate::{Config, MemberStateMutex};

/// Provide the ability of read the group name out of the [`ExecutiveMapping`].
pub trait GroupName {
    fn group_name(executive_mapping: &ExecutiveMapping) -> &String;
}

/// A wrapper for executive role which should be used as request guards to check if a user has an executive role.
/// The parameter type represents the role it self.
/// Despite the requirement that the parameter type is public, this guard is type safe over the whole application, as the [`ExecutiveRole`] can only be constructed within this module due to the restriction of the inner field.
pub struct ExecutiveRole<G>(G)
where
    G: GroupName + Sized + Default;

/// A role which is able to read and write to the archive.
#[derive(Default, Debug)]
pub struct Archive();

impl GroupName for Archive {
    fn group_name(executive_mapping: &ExecutiveMapping) -> &String {
        &executive_mapping.archive
    }
}

#[rocket::async_trait]
impl<'r, G> FromRequest<'r> for ExecutiveRole<G>
where
    G: GroupName + Sized + Default,
{
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let conf = request
            .rocket()
            .state::<Config>()
            .expect("Application configuration");
        let member_state = request
            .rocket()
            .state::<MemberStateMutex>()
            .expect("member state");
        let executives = &member_state.read().await.executives;
        let member_outcome: Outcome<Member, ()> = Member::from_request(request).await;
        if let Success(member) = member_outcome {
            debug!("Request contains the member '{}'", member.full_username);
            let group_name = G::group_name(&conf.ldap.executive_mapping);
            let group = executives
                .iter()
                .find(|g| g.name_plural.eq_ignore_ascii_case(group_name));
            if group.is_some()
                && group
                    .expect("Executive group")
                    .members
                    .iter()
                    .any(|m| m.eq_ignore_ascii_case(member.full_username.as_str()))
            {
                Success(ExecutiveRole(G::default()))
            } else {
                warn!("Member '{}' is not member of the '{}' executive role or the group does not exist on the directory server", member.full_username, group_name);
                Forward(())
            }
        } else {
            debug!("Request does not contain a member");
            Forward(())
        }
    }
}

impl<'r, G> OpenApiFromRequest<'r> for ExecutiveRole<G>
where
    G: GroupName + Sized + Default,
{
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut security_req = SecurityRequirement::new();
        // Each security requirement needs to be met before access is allowed.
        security_req.insert("executive roles".to_owned(), Vec::new());
        Ok(RequestHeaderInput::Security(
            "executive roles".to_string(),
            SecurityScheme {
                description: Some("Required for requests which need executive roles provided by a bearer token. Log in first to retrieve it".to_string()),
                data: SecuritySchemeData::Http {
                    scheme: "bearer".to_string(),
                    bearer_format: Some("JWT".to_string()),
                },
                extensions: Object::default(),
            },
            security_req,
        ))
    }
}
