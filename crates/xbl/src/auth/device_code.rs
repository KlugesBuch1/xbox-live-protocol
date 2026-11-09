//! The device-code request and token polling.

use std::time::{Duration, Instant};

use reqwest::header::ACCEPT;
use serde::Deserialize;

use super::token::TokenResponse;
use super::{
    Auth, AuthError, CLIENT_ID, DEFAULT_DEVICE_CODE_LIFETIME, DEFAULT_POLL_INTERVAL,
    DEVICE_CODE_GRANT, DeviceCode, LiveToken, SCOPE, read_json,
};

/// The response from `oauth20_connect.srf`.
#[derive(Deserialize)]
struct DeviceCodeResponse {
    user_code: String,
    device_code: String,
    #[serde(alias = "verification_url")]
    verification_uri: String,
    #[serde(default = "default_interval")]
    interval: u64,
    #[serde(default = "default_lifetime")]
    expires_in: u64,
}

fn default_interval() -> u64 {
    DEFAULT_POLL_INTERVAL.as_secs()
}

fn default_lifetime() -> u64 {
    DEFAULT_DEVICE_CODE_LIFETIME.as_secs()
}

/// The error object that the token endpoint returns while polling.
#[derive(Deserialize)]
struct OAuthError {
    error: String,
}

impl Auth {
    /// Starts the device-code flow.
    ///
    /// The response contains a user code and a verification page. Show both to
    /// the user, then call [`Auth::poll_device_code`] with the returned value.
    ///
    /// Returns [`AuthError::Api`] if Microsoft rejects the request and
    /// [`AuthError::Transport`] if it cannot be sent.
    pub async fn request_device_code(&self) -> Result<DeviceCode, AuthError> {
        let response = self
            .client
            .post(&self.endpoints.device_code)
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", SCOPE),
                ("response_type", "device_code"),
            ])
            .send()
            .await?;
        let body: DeviceCodeResponse = read_json(response).await?;

        Ok(DeviceCode {
            device_code: body.device_code,
            user_code: body.user_code,
            verification_uri: body.verification_uri,
            interval: Duration::from_secs(body.interval),
            expires_in: Duration::from_secs(body.expires_in),
        })
    }

    /// Polls the token endpoint until the user finishes the sign-in.
    ///
    /// `authorization_pending` keeps the loop running. `slow_down` increases the
    /// poll interval by the amount the service asks for. An expired device code
    /// and a declined sign-in return [`AuthError::Auth`].
    ///
    /// Returns [`AuthError::Api`] for any other error response,
    /// [`AuthError::Transport`] on a network failure and [`AuthError::Decode`]
    /// for a body that is not the expected JSON.
    pub async fn poll_device_code(&self, code: &DeviceCode) -> Result<LiveToken, AuthError> {
        let started = Instant::now();
        let mut interval = code.interval;

        loop {
            let elapsed = started.elapsed();
            if elapsed >= code.expires_in {
                return Err(AuthError::Auth("the device code expired".to_owned()));
            }

            let remaining = code.expires_in.saturating_sub(elapsed);
            tokio::time::sleep(interval.min(remaining)).await;

            if started.elapsed() >= code.expires_in {
                return Err(AuthError::Auth("the device code expired".to_owned()));
            }

            let response = self
                .client
                .post(&self.endpoints.token)
                .header(ACCEPT, "application/json")
                .form(&[
                    ("client_id", CLIENT_ID),
                    ("device_code", code.device_code.as_str()),
                    ("grant_type", DEVICE_CODE_GRANT),
                ])
                .send()
                .await?;
            let status = response.status();
            let body = response.bytes().await?;

            if status.is_success() {
                let tokens: TokenResponse = serde_json::from_slice(&body)?;
                return Ok(tokens.into_live_token());
            }

            match oauth_error(&body).as_deref() {
                Some("authorization_pending") => {}
                Some("slow_down") => {
                    interval = interval.saturating_add(self.slow_down_increment);
                }
                Some("expired_token") => {
                    return Err(AuthError::Auth("the device code expired".to_owned()));
                }
                Some("access_denied" | "authorization_declined") => {
                    return Err(AuthError::Auth("the sign-in was declined".to_owned()));
                }
                _ => {
                    return Err(AuthError::Api {
                        status: status.as_u16(),
                        body: String::from_utf8_lossy(&body).into_owned(),
                    });
                }
            }
        }
    }
}

/// Reads the `error` field without keeping any other part of the body.
fn oauth_error(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<OAuthError>(body)
        .ok()
        .map(|error| error.error)
}
