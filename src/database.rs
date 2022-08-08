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

use std::error::Error;

use reqwest::{Client, ClientBuilder, Url};
use rocket::serde::Serialize;

use crate::{Config, DatabaseClient};

static KEG_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Initialize the database client and configures it.
/// If the initialization fails this function will panic.
/// After the initialization this functions tries to authenticate against the database interface using cookies.
/// When this fails, an error will be printed.
///
/// # Arguments
///
/// * `conf`: the application configuration
///
/// returns: the configured [`DatabaseClient`]
pub async fn initialize_client(conf: &Config) -> DatabaseClient {
    let client = ClientBuilder::new()
        .user_agent(KEG_USER_AGENT)
        .cookie_store(true)
        .build()
        .map_err(|e| {
            error!("Unable to initialize http client: {}", e);
            e
        })
        .expect("First database client");
    authenticate(conf, &client)
        .await
        .map_err(|e| {
            error!("Unable to authenticate http client: {}", e);
            e
        })
        .expect("First authenticated client");
    client
}

#[derive(Serialize)]
struct Credentials {
    name: String,
    password: String,
}

impl Credentials {
    fn from_config(conf: &Config) -> Self {
        Self {
            name: conf.database.username.to_string(),
            password: conf.database.password.to_string(),
        }
    }
}

/// The authentication function to perform an HTTP authentication request against the database server.
/// If the process was successful, the authentication cookie will be stored in the cookie store.
///
/// # Arguments
///
/// * `conf`: the application configuration
/// * `client`: the HTTP client to use, cookie support is required
///
/// returns: ()
pub(crate) async fn authenticate(conf: &Config, client: &Client) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(&*format!(
        "{}{}",
        conf.database.url, conf.database.database_mapping.authentication
    ))?;
    let request = client
        .post(url)
        .form(&Credentials::from_config(conf))
        .build()?;
    let response = client.execute(request).await?;
    response.error_for_status()?;
    info!("Authentication to the database interface was successful");
    Ok(())
}
