//! Device-code sign-in and the two signed Xbox Live auth requests.
//!
//! Auth-Chain:
//!
//! ```text
//! oauth20_connect.srf -> oauth20_token.srf (device code grant)
//!   -> device.auth.xboxlive.com/device/authenticate (signed)
//!   -> sisu.xboxlive.com/authorize (signed)
//!   -> Authorization: XBL3.0 x={uhs};{token}
//! ```
//!
//! [`Auth::request_device_code`] starts the flow. Show the returned
//! [`DeviceCode`] to the user, then call [`Auth::poll_device_code`] and
//! [`Auth::authenticate`].

mod device_code;
mod error;
mod sisu;
mod token;

#[cfg(test)]
mod tests;

use std::time::Duration;

use serde::de::DeserializeOwned;

pub use error::AuthError;
pub use token::{DeviceCode, LiveToken, XboxToken};

/// The public client id used by the Xbox app for the device-code flow.
const CLIENT_ID: &str = "000000004424da1f";
/// The scope that lets SISU exchange the Microsoft token for an Xbox token.
const SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";
/// The grant type for the device-code token poll.
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// The grant type for a refresh-token exchange.
const REFRESH_GRANT: &str = "refresh_token";
/// The relying party for the device token.
const AUTH_RELYING_PARTY: &str = "http://auth.xboxlive.com";
/// The relying party for the Xbox Live token.
const XBOX_RELYING_PARTY: &str = "http://xboxlive.com";
/// The site name that makes SISU produce an `XBL3.0` header.
const SITE_NAME: &str = "user.auth.xboxlive.com";
/// The sandbox for a normal account.
const SANDBOX: &str = "RETAIL";
/// The device type sent in the device token request.
const DEVICE_TYPE: &str = "Win32";
/// The client version sent in the device token request.
const CLIENT_VERSION: &str = "0.0.0";
/// The default poll interval when the service does not send one.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
/// The default device-code lifetime when the service does not send one.
const DEFAULT_DEVICE_CODE_LIFETIME: Duration = Duration::from_secs(900);
/// The interval increase that the service asks for with `slow_down`.
const SLOW_DOWN_INCREMENT: Duration = Duration::from_secs(5);

/// The four HTTP endpoints used by the auth chain.
#[derive(Clone, Debug)]
pub struct Endpoints {
    /// The `oauth20_connect.srf` URL that starts the device-code flow.
    pub device_code: String,
    /// The `oauth20_token.srf` URL that exchanges the device or refresh code.
    pub token: String,
    /// The `device/authenticate` URL that creates the device token.
    pub device_authenticate: String,
    /// The SISU `/authorize` URL that creates the Xbox Live token.
    pub sisu_authorize: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            device_code: "https://login.live.com/oauth20_connect.srf".to_owned(),
            token: "https://login.live.com/oauth20_token.srf".to_owned(),
            device_authenticate: "https://device.auth.xboxlive.com/device/authenticate".to_owned(),
            sisu_authorize: "https://sisu.xboxlive.com/authorize".to_owned(),
        }
    }
}

/// The entry point for the Xbox Live auth chain.
///
/// One `Auth` value can start several sign-ins. It holds a `reqwest::Client`
/// and no tokens. Use [`Auth::with_endpoints`] to point it at a test server.
#[derive(Clone, Debug)]
pub struct Auth {
    client: reqwest::Client,
    endpoints: Endpoints,
    slow_down_increment: Duration,
}

impl Default for Auth {
    fn default() -> Self {
        Self::new()
    }
}

impl Auth {
    /// Creates a client for the live Microsoft and Xbox Live endpoints.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoints: Endpoints::default(),
            slow_down_increment: SLOW_DOWN_INCREMENT,
        }
    }

    /// Creates a client for custom endpoints.
    #[must_use]
    pub fn with_endpoints(endpoints: Endpoints) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoints,
            slow_down_increment: SLOW_DOWN_INCREMENT,
        }
    }

    /// Shortens the `slow_down` increase. Only compiled for tests.
    #[cfg(test)]
    fn with_slow_down_increment(mut self, increment: Duration) -> Self {
        self.slow_down_increment = increment;
        self
    }
}

/// Reads a JSON response and turns a non-success status into [`AuthError::Api`].
async fn read_json<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, AuthError> {
    let status = response.status();
    let body = response.bytes().await?;

    if !status.is_success() {
        return Err(AuthError::Api {
            status: status.as_u16(),
            body: String::from_utf8_lossy(&body).into_owned(),
        });
    }

    Ok(serde_json::from_slice(&body)?)
}

/// Returns the path and query that a signed request must cover.
fn path_and_query(url: &str) -> Result<String, AuthError> {
    let url = reqwest::Url::parse(url)
        .map_err(|_| AuthError::Auth("the endpoint URL is invalid".to_owned()))?;
    let mut path = url.path().to_owned();

    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }

    Ok(path)
}
