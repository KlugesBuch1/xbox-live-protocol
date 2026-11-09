//! The two signed requests: device authentication and SISU authorization.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::token::{LiveToken, XboxToken};
use super::{
    AUTH_RELYING_PARTY, Auth, AuthError, CLIENT_ID, CLIENT_VERSION, DEVICE_TYPE, SANDBOX,
    SITE_NAME, XBOX_RELYING_PARTY, path_and_query, read_json,
};
use crate::signing::{FileTime, sign_request};

/// The public half of the per-relying-party P-256 key.
#[derive(Clone, Serialize)]
struct ProofKey {
    kty: &'static str,
    crv: &'static str,
    alg: &'static str,
    #[serde(rename = "use")]
    r#use: &'static str,
    x: String,
    y: String,
}

impl ProofKey {
    /// Serializes the public key as the JWK that the auth services expect.
    ///
    /// The uncompressed SEC1 point is 65 bytes: one tag byte followed by the
    /// 32-byte x and y coordinates. Encoding those slices keeps every
    /// coordinate at 32 bytes, even when it starts with a zero byte.
    fn new(key: &SigningKey) -> Self {
        let point = key.verifying_key().to_sec1_point(false);
        let bytes = point.as_bytes();
        debug_assert_eq!(bytes.len(), 65, "an uncompressed P-256 point");
        debug_assert_eq!(bytes[0], 0x04, "an uncompressed P-256 point");

        Self {
            kty: "EC",
            crv: "P-256",
            alg: "ES256",
            r#use: "sig",
            x: URL_SAFE_NO_PAD.encode(&bytes[1..33]),
            y: URL_SAFE_NO_PAD.encode(&bytes[33..65]),
        }
    }
}

/// The body of `POST device/authenticate`.
#[derive(Serialize)]
struct DeviceAuthRequest {
    #[serde(rename = "Properties")]
    properties: DeviceProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: &'static str,
    #[serde(rename = "TokenType")]
    token_type: &'static str,
}

/// The `Properties` object of the device token request.
#[derive(Serialize)]
struct DeviceProperties {
    #[serde(rename = "AuthMethod")]
    auth_method: &'static str,
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "DeviceType")]
    device_type: &'static str,
    #[serde(rename = "SerialNumber")]
    serial_number: String,
    #[serde(rename = "Version")]
    version: &'static str,
    #[serde(rename = "ProofKey")]
    proof_key: ProofKey,
}

/// The body of `POST sisu/authorize`.
#[derive(Serialize)]
struct SisuRequest {
    #[serde(rename = "AccessToken")]
    access_token: String,
    #[serde(rename = "AppId")]
    app_id: &'static str,
    #[serde(rename = "DeviceToken")]
    device_token: String,
    #[serde(rename = "Sandbox")]
    sandbox: &'static str,
    #[serde(rename = "UseModernGamertag")]
    use_modern_gamertag: bool,
    #[serde(rename = "SiteName")]
    site_name: &'static str,
    #[serde(rename = "RelyingParty")]
    relying_party: &'static str,
    #[serde(rename = "ProofKey")]
    proof_key: ProofKey,
}

/// The response of `POST device/authenticate`.
#[derive(Deserialize)]
struct DeviceTokenResponse {
    #[serde(rename = "Token")]
    token: String,
}

/// The response of `POST sisu/authorize`.
#[derive(Deserialize)]
struct SisuResponse {
    #[serde(rename = "AuthorizationToken")]
    authorization_token: SisuToken,
}

#[derive(Deserialize)]
struct SisuToken {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    #[serde(rename = "Xui", alias = "xui")]
    xui: Vec<Xui>,
}

#[derive(Deserialize)]
struct Xui {
    uhs: String,
}

impl Auth {
    /// Creates a device token and exchanges the Microsoft token at SISU.
    ///
    /// A fresh P-256 key is generated for this relying party. The same key
    /// signs both requests, because the device token is bound to the proof key
    /// in the SISU request. A second relying party needs its own call and its
    /// own key.
    ///
    /// Returns [`AuthError::Api`] if either service rejects the request,
    /// [`AuthError::Transport`] on a network failure, [`AuthError::Decode`] for
    /// an unexpected body and [`AuthError::Auth`] when the OS random source or
    /// the SISU response is unusable.
    pub async fn authenticate(&self, live: &LiveToken) -> Result<XboxToken, AuthError> {
        let signing_key = SigningKey::try_generate()
            .map_err(|_| AuthError::Auth("the operating system random source failed".to_owned()))?;
        let proof_key = ProofKey::new(&signing_key);

        let device_body = serde_json::to_vec(&DeviceAuthRequest {
            properties: DeviceProperties {
                auth_method: "ProofOfPossession",
                id: random_guid()?,
                device_type: DEVICE_TYPE,
                serial_number: random_guid()?,
                version: CLIENT_VERSION,
                proof_key: proof_key.clone(),
            },
            relying_party: AUTH_RELYING_PARTY,
            token_type: "JWT",
        })?;
        let device_path = path_and_query(&self.endpoints.device_authenticate)?;
        let device_signature = sign_request(
            &signing_key,
            FileTime::now(),
            "POST",
            &device_path,
            &device_body,
        );

        let response = self
            .client
            .post(&self.endpoints.device_authenticate)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .header("x-xbl-contract-version", "1")
            .header("Signature", device_signature)
            .body(device_body)
            .send()
            .await?;
        let device_token: DeviceTokenResponse = read_json(response).await?;

        let sisu_body = serde_json::to_vec(&SisuRequest {
            access_token: format!("t={}", live.access_token()),
            app_id: CLIENT_ID,
            device_token: device_token.token,
            sandbox: SANDBOX,
            use_modern_gamertag: true,
            site_name: SITE_NAME,
            relying_party: XBOX_RELYING_PARTY,
            proof_key,
        })?;
        let sisu_path = path_and_query(&self.endpoints.sisu_authorize)?;
        let sisu_signature = sign_request(
            &signing_key,
            FileTime::now(),
            "POST",
            &sisu_path,
            &sisu_body,
        );

        let response = self
            .client
            .post(&self.endpoints.sisu_authorize)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .header("Signature", sisu_signature)
            .body(sisu_body)
            .send()
            .await?;
        let sisu: SisuResponse = read_json(response).await?;

        let SisuResponse {
            authorization_token,
        } = sisu;
        let SisuToken {
            token,
            display_claims,
        } = authorization_token;
        let uhs = display_claims
            .xui
            .into_iter()
            .next()
            .map(|xui| xui.uhs)
            .ok_or_else(|| AuthError::Auth("SISU returned no user hash".to_owned()))?;

        Ok(XboxToken::new(token, uhs))
    }
}

/// Returns a random v4 GUID in the braces that the service expects.
fn random_guid() -> Result<String, AuthError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| AuthError::Auth("the operating system random source failed".to_owned()))?;

    // RFC 4122 version 4 and variant bits.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(format!(
        "{{{}-{}-{}-{}-{}}}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}
