//! Xbox Live authentication, proof-of-possession signing and profile requests.
//!
//! The protocol notes and the endpoint catalogue live in the repository:
//! <https://github.com/KlugesBuch1/xbox-live-protocol>
#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod auth;
pub mod client;
pub mod error;
pub mod profile;
pub mod signing;

pub use error::Error;
