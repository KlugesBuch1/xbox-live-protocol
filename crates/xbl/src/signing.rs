//! Proof-of-possession signing for Xbox Live auth requests.
//!
//! Two requests are signed: `POST device.auth.xboxlive.com/device/authenticate`
//! and `POST sisu.xboxlive.com/authorize`. The signature covers the method, the
//! path with its query, and the raw body bytes. The byte layout and the
//! measured failures are documented in the repository:
//! <https://github.com/KlugesBuch1/xbox-live-protocol/blob/main/docs/signing.md>
//!
//! The signed payload and the `Signature` header use different layouts: the
//! payload has separator bytes between its fields, the header does not.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use p256::ecdsa::signature::Signer;

/// A P-256 ECDSA signature. `to_bytes()` returns the raw `r || s` form that
/// Xbox Live expects.
pub use p256::ecdsa::Signature;
/// A P-256 ECDSA signing key.
pub use p256::ecdsa::SigningKey;

/// Number of 100-nanosecond ticks in one second.
const TICKS_PER_SECOND: u64 = 10_000_000;
/// Seconds between the Windows epoch (1601-01-01) and the Unix epoch (1970-01-01).
const WINDOWS_EPOCH_OFFSET_SECONDS: u64 = 11_644_473_600;
/// Policy version that the current client puts in the payload and header.
const POLICY_VERSION: u32 = 1;
/// Length of the `Signature` header block before base64.
const SIGNATURE_HEADER_LEN: usize = 4 + 8 + 64;

/// A Win32 FILETIME timestamp: 100-nanosecond ticks since 1 January 1601 UTC.
///
/// The server does not check this value against the clock. A wrong timestamp
/// is accepted as long as the same bytes are signed and sent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileTime(u64);

impl FileTime {
    /// Creates a timestamp from raw FILETIME ticks.
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Creates a timestamp from Unix seconds.
    ///
    /// The conversion adds the 11,644,473,600 seconds between the Windows and
    /// Unix epochs, then multiplies by 10,000,000 ticks per second.
    pub const fn from_unix_seconds(seconds: u64) -> Self {
        Self((seconds + WINDOWS_EPOCH_OFFSET_SECONDS) * TICKS_PER_SECOND)
    }

    /// Returns the current UTC time with one-second precision.
    ///
    /// The server does not compare this value with its own clock. A timestamp
    /// that is a little off is accepted as long as the same bytes are signed
    /// and sent.
    #[must_use]
    pub fn now() -> Self {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        Self::from_unix_seconds(seconds)
    }

    /// Returns the raw FILETIME ticks.
    pub const fn ticks(self) -> u64 {
        self.0
    }
}

/// Builds the bytes covered by a proof-of-possession signature.
///
/// The layout is:
///
/// ```text
/// version (4, big-endian) | 0x00 | FILETIME (8, big-endian) | 0x00 |
/// method | 0x00 | path and query | 0x00 0x00 | body | 0x00
/// ```
///
/// The body is copied byte for byte. Do not parse and re-serialize it before
/// signing.
pub struct PayloadBuilder<'a> {
    filetime: FileTime,
    method: &'a str,
    path_and_query: &'a str,
    body: &'a [u8],
}

impl<'a> PayloadBuilder<'a> {
    /// Creates a builder for one request.
    ///
    /// `method` is the HTTP method, for example `POST`. `path_and_query` is the
    /// request target with its query, for example `/authorize?x=1`, exactly as
    /// it is sent on the wire (including %-encoding).
    pub const fn new(
        filetime: FileTime,
        method: &'a str,
        path_and_query: &'a str,
        body: &'a [u8],
    ) -> Self {
        Self {
            filetime,
            method,
            path_and_query,
            body,
        }
    }

    /// Returns the bytes to sign. Pass them to the signer unchanged.
    #[must_use]
    pub fn build(&self) -> Vec<u8> {
        let method = self.method.as_bytes();
        let path = self.path_and_query.as_bytes();
        let mut payload = Vec::with_capacity(
            4 + 1 + 8 + 1 + method.len() + 1 + path.len() + 2 + self.body.len() + 1,
        );

        payload.extend_from_slice(&POLICY_VERSION.to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(&self.filetime.ticks().to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(method);
        payload.push(0);
        payload.extend_from_slice(path);
        payload.extend_from_slice(&[0, 0]);
        payload.extend_from_slice(self.body);
        payload.push(0);
        payload
    }
}

/// Encodes the 76-byte `Signature` header block as standard base64.
///
/// The block is the policy version (4 bytes, big-endian), the [`FileTime`]
/// (8 bytes, big-endian), and the raw `r || s` signature (64 bytes). The
/// version and the timestamp follow each other without separator bytes; the
/// signed payload does have separators there.
///
/// 76 bytes become 104 characters ending in `==`. Standard base64 with padding
/// is required. base64url returns 401.
#[must_use]
pub fn encode_signature_header(filetime: FileTime, signature: &Signature) -> String {
    let mut block = [0_u8; SIGNATURE_HEADER_LEN];
    block[..4].copy_from_slice(&POLICY_VERSION.to_be_bytes());
    block[4..12].copy_from_slice(&filetime.ticks().to_be_bytes());
    block[12..].copy_from_slice(&signature.to_bytes());
    STANDARD.encode(block)
}

/// Signs one request and returns the value for the `Signature` header
///
/// This builds the payload with [`PayloadBuilder`], signs it with ES256, and
/// encodes the header with [`encode_signature_header`]. The body is signed as
/// the exact bytes passed in.
#[must_use]
pub fn sign_request(
    key: &SigningKey,
    filetime: FileTime,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> String {
    let payload = PayloadBuilder::new(filetime, method, path_and_query, body).build();
    let signature: Signature = key.sign(&payload);
    encode_signature_header(filetime, &signature)
}

#[cfg(test)]
mod tests {
    use super::{
        FileTime, PayloadBuilder, SIGNATURE_HEADER_LEN, Signature, SigningKey,
        encode_signature_header, sign_request,
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use p256::ecdsa::signature::Verifier;

    // Timestamp 1, POST /path, body {}: the offsets from the repository notes.
    #[test]
    fn payload_matches_the_documented_layout() {
        let payload = PayloadBuilder::new(FileTime::from_ticks(1), "POST", "/path", b"{}").build();

        let expected =
            b"\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00POST\x00/path\x00\x00{}\x00";
        assert_eq!(payload, expected);
        assert_eq!(payload.len(), 29);
    }

    #[test]
    fn byte_13_separator_decides_200_or_403() {
        let payload = PayloadBuilder::new(FileTime::from_ticks(1), "POST", "/path", b"{}").build();

        assert_eq!(payload[13], 0);
        assert_eq!(&payload[13..18], b"\0POST");
    }

    #[test]
    fn signature_header_block_is_76_bytes() {
        let signature = Signature::from_slice(&[0xaa; 64]).expect("valid test signature");
        let header = encode_signature_header(FileTime::from_ticks(1), &signature);
        let block = STANDARD.decode(header).expect("standard base64");

        assert_eq!(block.len(), SIGNATURE_HEADER_LEN);
        assert_eq!(SIGNATURE_HEADER_LEN, 76);
        assert_eq!(&block[..4], &[0, 0, 0, 1]);
        assert_eq!(&block[4..12], &1_u64.to_be_bytes());
        assert_eq!(&block[12..], &[0xaa; 64]);
    }

    #[test]
    fn signature_header_uses_standard_base64_with_padding() {
        let signature = Signature::from_slice(&[0xaa; 64]).expect("valid test signature");
        let header = encode_signature_header(FileTime::from_ticks(1), &signature);

        assert!(!header.contains('-'), "base64url alphabet");
        assert!(!header.contains('_'), "base64url alphabet");
        assert!(header.ends_with("=="), "padding is kept");
        assert_eq!(header.len(), 104);
    }

    #[test]
    fn filetime_now_tracks_the_system_clock() {
        let now = FileTime::now();

        assert!(
            now.ticks() > FileTime::from_unix_seconds(1_600_000_000).ticks(),
            "the test clock is after 2020"
        );
        assert!(
            now.ticks() < FileTime::from_unix_seconds(4_102_444_800).ticks(),
            "the test clock is before 2100"
        );
        assert_eq!(now.ticks() % 10_000_000, 0, "one-second precision");
    }

    #[test]
    fn filetime_counts_ticks_from_1601() {
        assert_eq!(
            FileTime::from_unix_seconds(0).ticks(),
            116_444_736_000_000_000
        );
        assert_eq!(
            FileTime::from_unix_seconds(1).ticks(),
            116_444_736_010_000_000
        );
    }

    #[test]
    fn es256_roundtrip_with_fixed_test_key() {
        let key = SigningKey::from_slice(&[0x01; 32]).expect("test key is valid");
        let filetime = FileTime::from_unix_seconds(1_700_000_000);
        let method = "POST";
        let path = "/authorize?x=1";
        let body = br#"{ "b": 2, "a": 1 }"#;

        let payload = PayloadBuilder::new(filetime, method, path, body).build();
        assert!(payload.ends_with(b"{ \"b\": 2, \"a\": 1 }\0"));

        let header = sign_request(&key, filetime, method, path, body);
        let block = STANDARD.decode(header).expect("standard base64");
        assert_eq!(block.len(), SIGNATURE_HEADER_LEN);
        assert_eq!(&block[..4], &[0, 0, 0, 1]);
        assert_eq!(&block[4..12], &filetime.ticks().to_be_bytes());

        let signature = Signature::from_slice(&block[12..]).expect("64-byte signature");
        key.verifying_key()
            .verify(&payload, &signature)
            .expect("signature verifies");
    }
}
