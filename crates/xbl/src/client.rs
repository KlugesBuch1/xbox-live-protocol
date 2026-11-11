//! The HTTP client for the Xbox Live profile host.
//!
//! The client holds [`XboxToken`] and sends it as the `Authorization`
//! header on every request. The base URL can be replaced so the tests run
//! against a local server.

use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde::de::DeserializeOwned;

use crate::auth::XboxToken;
use crate::error::Error;

/// The host that serves the profile routes.
const DEFAULT_BASE_URL: &str = "https://profile.xboxlive.com";

/// A client that sends Xbox Live requests with one account's token.
#[derive(Clone, Debug)]
pub struct Client {
    http: reqwest::Client,
    token: XboxToken,
    base_url: String,
}

impl Client {
    /// Creates a client for the live profile host.
    #[must_use]
    pub fn new(token: XboxToken) -> Self {
        Self::with_base_url(token, DEFAULT_BASE_URL)
    }

    /// Creates a client for another host, for example a test server.
    #[must_use]
    pub fn with_base_url(token: XboxToken, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            token,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        }
    }

    /// Sends a GET request to `path` and decodes the JSON body.
    ///
    /// `path` starts with `/` and includes the query. The request carries the
    /// XBL3.0 `Authorization` header and the `X-Xbl-Contract-Version` value.
    /// A non-success status becomes [`Error::Api`].
    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        contract: &str,
    ) -> Result<T, Error> {
        let response = self
            .http
            .get(format!("{}{path}", self.base_url))
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, self.token.authorization_header())
            .header("X-Xbl-Contract-Version", contract)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                body: String::from_utf8_lossy(&body).into_owned(),
            });
        }

        Ok(serde_json::from_slice(&body)?)
    }
}
