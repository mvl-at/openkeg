// OpenKeg, the lightweight backend of the Musikverein Leopoldsdorf.
// Copyright (C) 2022-2023  Richard Stöckl
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

/// The application configuration.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The configuration of the directory server.
    pub ldap: LdapConfig,
    /// The configuration of the jwts.
    pub jwt: JwtConfig,
    /// The configuration of the certificates.
    pub cert: CertConfig,
    /// The configuration of the database.
    pub database: DatabaseConfig,
    /// The url to use for a server entry in the OpenApi schema.
    /// It is highly recommended to use a URL to this server instance.
    pub openapi_url: String,
    /// Whether expose a directory to the public or not.
    /// May be used to serve the swagger ui or the RapiDoc.
    pub serve_static_directory: bool,
    /// The filesystem path to the public directory.
    pub static_directory_path: String,
    /// The URL where to mount the public directory.
    pub static_directory_url: String,
    /// The configuration for the document server.
    pub document_server: DocumentServer,
    /// The configuration for the calendar.
    pub calendar: CalendarConfig,
}

/// The configuration of the directory server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LdapConfig {
    /// The server url.
    pub server: String,
    /// The dn to use to bind to the server.
    /// If 'None' a bind without a user will be tried.
    pub dn: Option<String>,
    /// The password for the dn.
    pub password: Option<String>,
    /// The synchronization interval for the member and groups in *seconds*.
    pub synchronization_interval: u64,
    /// The base dn where to start to search for member.
    pub member_base: String,
    /// The filter to use to search member.
    pub member_filter: String,
    /// The base dn where to start to search for sutlers.
    pub sutler_base: String,
    /// The filter to use to search sutlers.
    pub sutler_filter: String,
    /// The base dn where to start to search for honoraries.
    pub honorary_base: String,
    /// The filter to use to search honoraries.
    pub honorary_filter: String,
    /// The base dn where to start to search for registers.
    pub register_base: String,
    /// The filter to use to search registers.
    pub register_filter: String,
    /// The base dn where to start to search for executives.
    pub executives_base: String,
    /// The filter to use to search executives.
    pub executives_filter: String,
    /// The mapping for the member attributes.
    pub member_mapping: MemberMapping,
    /// The mapping for the address attributes.
    pub address_mapping: AddressMapping,
    /// The mapping for the group attributes.
    pub group_mapping: GroupMapping,
    /// The mapping of the executive roles.
    pub executive_mapping: ExecutiveMapping,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            server: "auth://localhost:389".to_string(),
            dn: None,
            password: None,
            synchronization_interval: 300,
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
            member_mapping: Default::default(),
            address_mapping: Default::default(),
            group_mapping: Default::default(),
            executive_mapping: Default::default(),
        }
    }
}

/// The mapping to member.
/// This refers to an LDAP structure which likely has the 'mvlMember' object class.
/// The attribute descriptions refer to the content of the object attribute and provide an example often seen for the mapping.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemberMapping {
    /// The short username mapping, something like 'cn' or 'uid'.
    pub username: String,
    /// The full username mapping, normally 'dn'.
    pub full_username: String,
    /// The first name mapping such as 'givenName'.
    pub first_name: String,
    /// The last name mapping such as 'sn'.
    pub last_name: String,
    /// The common name, this is normally how someone wants to be called.
    /// Normally 'cn'.
    pub common_name: String,
    /// Whether this member uses the WhatsApp services or not.
    /// Something like 'wa'.
    pub whatsapp: String,
    /// The year when this member joined the society.
    /// Something like 'joining'.
    pub joining: String,
    /// Whether this member is listed publicly or not.
    /// Something like 'listed'.
    pub listed: String,
    /// Whether this member is registered at the ÖBV or not.
    /// Something like 'official'.
    pub official: String,
    /// The gender of this member.
    /// Something like 'gender'.
    pub gender: String,
    /// Whether this member is active or not.
    /// Something like 'active'.
    pub active: String,
    /// The mobile number of this member.
    /// Normally 'mobile'
    pub mobile: String,
    /// The date of birth of this member.
    /// Something like 'birthday'.
    pub birthday: String,
    /// The email address of this member.
    /// Normally 'mail'.
    pub mail: String,
    /// The titles of this member such as 'Kapellmeister'.
    /// Normally 'title'.
    pub titles: String,
    /// The photo of this member.
    /// Normally 'jpegPhoto'
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
            titles: "title".to_string(),
            photo: "jpegPhoto".to_string(),
        }
    }
}

/// The mapping to addresses.
/// This refers to an LDAP structure which supports addresses.
/// The attribute descriptions refer to the content of the object attribute and provide an example often seen for the mapping.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressMapping {
    /// The street of the address such as 'Lasseer Strasse'.
    /// Normally 'street'.
    pub street: String,
    /// The number of the address such as '3' or '3a'.
    /// Normally 'houseIdentifier'.
    pub house_number: String,
    /// The postal code of this address such as '2285'.
    /// Normally 'postalCode'.
    pub postal_code: String,
    /// The city of this address such as 'Leopoldsdorf i.M.'.
    /// Normally 'l'.
    pub city: String,
    /// The state of this address such 'Burgenland'.
    /// Normally 'st'.
    pub state: String,
    /// The country code of this address such as 'AT'.
    /// Normally 'st'.
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

/// The mapping to groups.
/// These mappings are valid for LDAP structures which likely require 'mvlGroup' as object class.
/// The attribute descriptions refer to the content of the object attribute and provide an example often seen for the mapping.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMapping {
    /// The plural name of this group.
    /// This may be a register name such as 'Flügelhörner' or another member group such as 'Sutlers' or 'Honoraries'.
    /// Something like 'cn'.
    pub name_plural: String,
    /// The singular name of this group.
    /// This may be a register name such as 'Flügelhorn' or another member group such as 'Sutler' or 'Honorary'.
    /// Normally 'cns'.
    pub name: String,
    /// The description of this group.
    /// Normally 'description'.
    pub description: String,
    /// The member dns of this group.
    /// Normally 'member'.
    pub members: String,
}

impl Default for GroupMapping {
    fn default() -> Self {
        Self {
            name_plural: "cn".to_string(),
            name: "cns".to_string(),
            description: "description".to_string(),
            members: "member".to_string(),
        }
    }
}

/// The mapping of all the executive roles.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutiveMapping {
    /// Role to manage the archive, both reading and writing.
    pub archive: String,
}

impl Default for ExecutiveMapping {
    fn default() -> Self {
        Self {
            archive: "".to_string(),
        }
    }
}

/// The configuration section about jwts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    /// The expiration of request tokens given in *minutes*.
    pub expiration: i64,
    /// The expiration of the refresh tokens given in *hours*.
    pub renewal_expiration: i64,
    /// The issuer used for token generation
    pub issuer: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            expiration: 2 * 60,
            renewal_expiration: 365 * 24,
            issuer: "keg".to_string(),
        }
    }
}

/// The configuration for the certificates.
/// These are mostly used for signing and checking jwts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertConfig {
    /// The path to the private key in the pem format
    pub private_key_path: String,
    /// The path to the public key in the pem format
    pub public_key_path: String,
}

impl Default for CertConfig {
    fn default() -> Self {
        Self {
            private_key_path: "keg-private-key.pem".to_string(),
            public_key_path: "keg-public-key.pem".to_string(),
        }
    }
}

/// The configuration of the database connection.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// The base url to the CouchDB Rest interface
    pub url: String,
    /// The username of the CouchDB user
    pub username: String,
    /// The password of the CouchDB user
    pub password: String,
    /// The score partition prefix
    pub score_partition: String,
    /// The database url mappings
    pub database_mapping: DatabaseMapping,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            score_partition: "scores".to_string(),
            database_mapping: Default::default(),
        }
    }
}

/// A holder for the database mappings.
/// These are a bunch of strings which define the urls where to retrieve and store data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMapping {
    /// The endpoint used for authentication
    pub authentication: String,
    /// The endpoint which returns all scores which are available, requires the ability of sorting
    pub all_scores: String,
    /// The endpoint to search for scores
    pub find_scores: String,
    /// The endpoint to receive a single score by its id
    pub get_score: String,
    /// The endpoint to put a single score
    pub put_score: String,
    /// The endpoint to delete a single score by its id and revision
    pub delete_score: String,
    /// The endpoint for the genres count statistic.
    pub genres_statistic: String,
    /// The endpoint for the composers count statistic.
    pub composers_statistic: String,
    /// The endpoint for the arrangers count statistic.
    pub arrangers_statistic: String,
    /// The endpoint for the publishers count statistic.
    pub publishers_statistic: String,
    /// The endpoint for the books count statistic.
    pub books_statistic: String,
    /// The endpoint for the locations count statistic.
    pub locations_statistic: String,
}

impl Default for DatabaseMapping {
    fn default() -> Self {
        Self {
            authentication: "/_session".to_string(),
            all_scores: "".to_string(),
            find_scores: "".to_string(),
            get_score: "".to_string(),
            put_score: "".to_string(),
            delete_score: "".to_string(),
            genres_statistic: "".to_string(),
            composers_statistic: "".to_string(),
            arrangers_statistic: "".to_string(),
            publishers_statistic: "".to_string(),
            books_statistic: "".to_string(),
            locations_statistic: "".to_string(),
        }
    }
}

/// Configuration of the document server which provides all the documents for access.
/// The server must implement the WebDav specification.
/// In the context of a music society, this is typically a nextcloud instance.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentServer {
    /// The root of all documents.
    /// Must be a full qualified HTTP URL to the WebDav instance.
    /// May already contain directories.
    pub base_url: String,
    /// The mappings of the document types to server directories.
    pub mapping: DocumentMapping,
}

impl Default for DocumentServer {
    fn default() -> Self {
        Self {
            base_url: "".to_string(),
            mapping: Default::default(),
        }
    }
}

/// The mappings for the document types.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentMapping {
    /// The URL to the blackboard documents directory.
    pub blackboard: String,
    /// The URL to the blackboard assets directory.
    pub blackboard_assets: String,
}

impl Default for DocumentMapping {
    fn default() -> Self {
        Self {
            blackboard: "".to_string(),
            blackboard_assets: "".to_string(),
        }
    }
}

/// The configuration related to calendar.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarConfig {
    /// The URL to the ical which contains all events to show to the public.
    pub ical_url: String,
    /// The URL to the ical which contains all events which are for internal usage only such as preparations.
    pub ical_internal_url: String,
} 

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            ical_url: "".to_string(),
            ical_internal_url: "".to_string(),
        }
    }
}

/// Read the configuration from `keg.toml` and set the `KEG_` prefix for all rocket related environment variables.
/// Furthermore, the profile will be selected.
/// Note, that the functionality to specify another `keg.toml` path via the `KEG_CONFIG` environment variable is currently broken.
///
/// returns: Figment for the configuration  
pub fn read_config() -> Figment {
    Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("keg.toml").nested())
        .merge(Env::prefixed("KEG_").global())
        .select(Profile::from_env_or("KEG_PROFILE", "default"))
}
