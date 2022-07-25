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

use reqwest::{Client, ClientBuilder, Url};
use rocket::serde::Serialize;

use crate::{Config, DatabaseClient};

static KEG_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Initialize the database client and configures it.
/// If the initialization fails this function will panic.
/// After the initialization this functions tries to authenticate against the database interface using cookies.
/// When this fails, a warning will be printed.
///
/// # Arguments
///
/// * `conf`: the application configuration
///
/// returns: the configured [`DatabaseClient`]
pub async fn initialize_client(conf: &Config) -> DatabaseClient {
    let client_result = ClientBuilder::new()
        .user_agent(KEG_USER_AGENT)
        .cookie_store(true)
        .build();
    if client_result.is_err() {
        error!(
            "unable to initialize http client: {}",
            client_result.err().unwrap()
        );
        panic!();
    }
    let client = client_result.unwrap();
    authenticate(conf, &client)
        .await
        .expect("authenticated client");
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
pub(crate) async fn authenticate(conf: &Config, client: &Client) -> Result<(), ()> {
    let url_result = Url::parse(&*format!(
        "{}{}",
        conf.database.url, conf.database.database_mapping.authentication
    ));
    if url_result.is_err() {
        warn!(
            "unable to parse the authentication url: {}",
            url_result.err().unwrap()
        );
        return Err(());
    }
    let request_result = client
        .post(url_result.unwrap())
        .form(&Credentials::from_config(conf))
        .build();
    if request_result.is_err() {
        warn!(
            "unable to build the authentication request: {}",
            request_result.err().unwrap()
        );
        return Err(());
    }
    let response_result = client.execute(request_result.unwrap()).await;
    if response_result.is_err() {
        warn!(
            "unable to execute authentication request: {}",
            response_result.err().unwrap()
        );
        return Err(());
    }
    let response = response_result.unwrap();
    if response.status().is_client_error() || response.status().is_server_error() {
        warn!(
            "unable to authenticate, the server returned: {}",
            response.status()
        );
        Err(())
    } else {
        info!("authentication to the database interface was successful");
        Ok(())
    }
}
