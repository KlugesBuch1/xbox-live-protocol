//! The error type shared by the client and the profile routes.

use std::fmt;

use crate::auth::AuthError;

/// The error returned by [`crate::client::Client`].
///
/// The `Debug` output redacts response bodies. Match on the variant if you need
/// the body of an [`Error::Api`] error.
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The auth chain failed before a request was sent.
    #[error(transparent)]
    Auth(#[from] AuthError),
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
    /// A JSON response body could not be decoded.
    #[error("could not decode JSON")]
    Decode(#[from] serde_json::Error),
    /// The service answered 200 with an empty `profileUsers` list.
    ///
    /// The profile route reports a lookup that found no account this way, so an
    /// empty list is a failed lookup and not a profile without settings.
    #[error("the response has an empty profileUsers list")]
    EmptyProfileUsers,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(error) => f.debug_tuple("Auth").field(error).finish(),
            Self::Transport(error) => f.debug_tuple("Transport").field(error).finish(),
            Self::Api { status, body } => f
                .debug_struct("Api")
                .field("status", status)
                .field("body", &format!("<{} bytes redacted>", body.len()))
                .finish(),
            Self::Decode(error) => f.debug_tuple("Decode").field(error).finish(),
            Self::EmptyProfileUsers => f.write_str("EmptyProfileUsers"),
        }
    }
}
