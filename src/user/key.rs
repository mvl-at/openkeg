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

use std::fs;
use std::io::Error;

use crate::Config;

/// The private key used in this application e.g. for jwt signing.
pub struct PrivateKey(pub(crate) Vec<u8>);

/// The public key used in this application e.g. for signature checks.
pub struct PublicKey(pub(crate) Vec<u8>);

/// Reads the private key from the file whose path is provided in the application configuration.
///
/// # Arguments
///
/// * `config`: the application configuration
///
/// returns: Result<PrivateKey, Error>
pub fn read_private_key(config: &Config) -> Result<PrivateKey, Error> {
    fs::read(&config.cert.private_key_path).map(|k| PrivateKey(k))
}

/// Reads the public key from the file whose path is provided in the application configuration.
///
/// # Arguments
///
/// * `config`: the application configuration
///
/// returns: Result<PublicKey, Error>
pub fn read_public_key(config: &Config) -> Result<PublicKey, Error> {
    fs::read(&config.cert.public_key_path).map(|k| PublicKey(k))
}
