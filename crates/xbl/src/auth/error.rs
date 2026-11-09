//! Errors returned by the auth chain.

use std::fmt;

/// The error returned by [`crate::auth::Auth`].
///
/// The `Debug` output never includes response bodies or tokens. Match on the
/// variant if you need the body of an [`AuthError::Api`] error.
#[derive(thiserror::Error)]
pub enum AuthError {
    /// The sign-in failed before or during the token exchange.
    #[error("authentication failed: {0}")]
    Auth(String),
    /// The request could not be sent or the response could not be read.
    #[error("network request failed")]
    Transport(#[from] reqwest::Error),
    /// The service answered with a non-success status.
    #[error("the API returned HTTP {status}")]
    Api {
        /// The HTTP status code.
        status: u16,
        /// The raw response body.
        body: String,
    },
    /// A JSON request or response body could not be encoded or decoded.
    #[error("could not decode JSON")]
    Decode(#[from] serde_json::Error),
}

impl fmt::Debug for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(message) => f.debug_tuple("Auth").field(message).finish(),
            Self::Transport(error) => f.debug_tuple("Transport").field(error).finish(),
            Self::Api { status, body } => f
                .debug_struct("Api")
                .field("status", status)
                .field("body", &format!("<{} bytes redacted>", body.len()))
                .finish(),
            Self::Decode(error) => f.debug_tuple("Decode").field(error).finish(),
        }
    }
}
