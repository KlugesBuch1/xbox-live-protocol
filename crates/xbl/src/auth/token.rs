//! Tokens and the public token types.

use std::fmt;
use std::time::Duration;

use reqwest::header::ACCEPT;
use serde::Deserialize;

use super::{Auth, AuthError, CLIENT_ID, REFRESH_GRANT, SCOPE, read_json};

/// The device-code response that the user has to act on.
///
/// The `device_code` is a credential and is not shown in `Debug`. Show
/// [`DeviceCode::user_code`] and [`DeviceCode::verification_uri`] instead.
#[derive(Clone)]
pub struct DeviceCode {
    pub(crate) device_code: String,
    pub(crate) user_code: String,
    pub(crate) verification_uri: String,
    pub(crate) interval: Duration,
    pub(crate) expires_in: Duration,
}

impl DeviceCode {
    /// The code that the user types into the verification page.
    #[must_use]
    pub fn user_code(&self) -> &str {
        &self.user_code
    }

    /// The page where the user enters [`DeviceCode::user_code`].
    #[must_use]
    pub fn verification_uri(&self) -> &str {
        &self.verification_uri
    }

    /// How long to wait between token polls, as requested by the service.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// How long the device code stays valid.
    #[must_use]
    pub const fn expires_in(&self) -> Duration {
        self.expires_in
    }
}

impl fmt::Debug for DeviceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceCode")
            .field("user_code", &self.user_code)
            .field("verification_uri", &self.verification_uri)
            .field("interval", &self.interval)
            .field("expires_in", &self.expires_in)
            .field("device_code", &"<redacted>")
            .finish()
    }
}

/// The Microsoft account tokens from `login.live.com/oauth20_token.srf`.
///
/// This type does not implement `Display`, and its `Debug` output redacts both
/// tokens.
#[derive(Clone)]
pub struct LiveToken {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: u64,
}

impl LiveToken {
    pub(crate) fn new(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: u64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
        }
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }

    /// The refresh token, if the service returned one.
    ///
    /// The refresh token is a credential. Store it only if you need to start a
    /// later session with [`crate::auth::Auth::refresh`].
    #[must_use]
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    /// How long the access token stays valid.
    #[must_use]
    pub const fn expires_in(&self) -> Duration {
        Duration::from_secs(self.expires_in)
    }
}

impl fmt::Debug for LiveToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveToken")
            .field("access_token", &"<redacted>")
            .field(
                "refresh_token",
                &if self.refresh_token.is_some() {
                    "<redacted>"
                } else {
                    "<absent>"
                },
            )
            .field("expires_in", &self.expires_in)
            .finish()
    }
}

/// The Xbox Live token and user hash from SISU.
///
/// Use [`XboxToken::authorization_header`] to build the `Authorization` value.
/// This type does not implement `Display`, and its `Debug` output redacts the
/// token.
#[derive(Clone)]
pub struct XboxToken {
    token: String,
    uhs: String,
}

impl XboxToken {
    pub(crate) fn new(token: String, uhs: String) -> Self {
        Self { token, uhs }
    }

    /// The user hash that belongs to the Xbox Live token.
    #[must_use]
    pub fn uhs(&self) -> &str {
        &self.uhs
    }

    /// The Xbox Live token.
    ///
    /// This is a bearer credential. Print at most a short prefix of it.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Builds the value for the `Authorization` header.
    #[must_use]
    pub fn authorization_header(&self) -> String {
        format!("XBL3.0 x={};{}", self.uhs, self.token)
    }
}

impl fmt::Debug for XboxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XboxToken")
            .field("uhs", &self.uhs)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// The token endpoint response for the device-code and refresh grants.
#[derive(Deserialize)]
pub(super) struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default = "default_token_lifetime")]
    expires_in: u64,
}

fn default_token_lifetime() -> u64 {
    3_600
}

impl TokenResponse {
    pub(super) fn into_live_token(self) -> LiveToken {
        LiveToken::new(self.access_token, self.refresh_token, self.expires_in)
    }
}

impl Auth {
    /// Exchanges a refresh token for a new Microsoft account token.
    ///
    /// This is the refresh grant, not a new SISU call. Pass the result to
    /// [`Auth::authenticate`] to get a fresh Xbox Live token. The refresh token
    /// is a credential and is not stored by this client.
    ///
    /// Returns [`AuthError::Api`] when the token endpoint rejects the refresh
    /// token, [`AuthError::Transport`] on a network failure and
    /// [`AuthError::Decode`] for an unexpected body.
    pub async fn refresh(&self, refresh_token: &str) -> Result<LiveToken, AuthError> {
        let response = self
            .client
            .post(&self.endpoints.token)
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("grant_type", REFRESH_GRANT),
                ("refresh_token", refresh_token),
                ("scope", SCOPE),
            ])
            .send()
            .await?;
        let tokens: TokenResponse = read_json(response).await?;

        Ok(tokens.into_live_token())
    }
}
