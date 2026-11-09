//! Offline tests for the whole auth chain.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Verifier;
use serde_json::json;
use wiremock::matchers::{body_string_contains, header_exists, method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

use super::{Auth, AuthError, Endpoints, path_and_query};
use crate::signing::{FileTime, PayloadBuilder, Signature};

fn json_response(status: u16, body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(body)
}

fn endpoints(server: &MockServer) -> Endpoints {
    let base = server.uri();

    Endpoints {
        device_code: format!("{base}/oauth20_connect.srf"),
        token: format!("{base}/oauth20_token.srf"),
        device_authenticate: format!("{base}/device/authenticate"),
        sisu_authorize: format!("{base}/sisu/authorize"),
    }
}

fn test_auth(server: &MockServer) -> Auth {
    Auth::with_endpoints(endpoints(server)).with_slow_down_increment(Duration::ZERO)
}

/// Returns responses in order, one per matching request.
struct Sequence {
    responses: Mutex<VecDeque<ResponseTemplate>>,
}

impl Sequence {
    fn new(responses: Vec<ResponseTemplate>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
        }
    }
}

impl Respond for Sequence {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        self.responses
            .lock()
            .expect("the response queue is not poisoned")
            .pop_front()
            .expect("the token response queue is exhausted")
    }
}

async fn mount_device_code(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/oauth20_connect.srf"))
        .and(body_string_contains("client_id=000000004424da1f"))
        .and(body_string_contains("response_type=device_code"))
        .and(body_string_contains("MBI_SSL"))
        .respond_with(json_response(
            200,
            json!({
                "user_code": "USER-CODE",
                "device_code": "device-code-value",
                "verification_uri": "https://example.test/device",
                "expires_in": 900,
                "interval": 0
            }),
        ))
        .mount(server)
        .await;
}

async fn mount_token(server: &MockServer, response: impl Respond + 'static) {
    Mock::given(method("POST"))
        .and(path("/oauth20_token.srf"))
        .respond_with(response)
        .mount(server)
        .await;
}

async fn mount_device_auth(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path("/device/authenticate"))
        .and(header_exists("Signature"))
        .and(header_exists("x-xbl-contract-version"))
        .respond_with(response)
        .mount(server)
        .await;
}

async fn mount_sisu(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path("/sisu/authorize"))
        .and(header_exists("Signature"))
        .respond_with(response)
        .mount(server)
        .await;
}

fn token_success() -> ResponseTemplate {
    json_response(
        200,
        json!({
            "access_token": "msa-access-token",
            "refresh_token": "msa-refresh-token",
            "expires_in": 3600
        }),
    )
}

fn device_auth_success() -> ResponseTemplate {
    json_response(200, json!({ "Token": "device-token-value" }))
}

fn sisu_success() -> ResponseTemplate {
    json_response(
        200,
        json!({
            "AuthorizationToken": {
                "Token": "xbox-token-value",
                "DisplayClaims": {
                    "xui": [{ "uhs": "1234567890" }]
                }
            }
        }),
    )
}

fn body_text(request: &wiremock::Request) -> &str {
    std::str::from_utf8(&request.body).expect("the form body is UTF-8")
}

async fn recorded(server: &MockServer, path: &str) -> Vec<wiremock::Request> {
    server
        .received_requests()
        .await
        .expect("request recording is enabled")
        .into_iter()
        .filter(|request| request.url.path() == path)
        .collect()
}

fn verify_signed_request(request: &wiremock::Request, proof_key: &serde_json::Value) {
    let x = URL_SAFE_NO_PAD
        .decode(proof_key["x"].as_str().expect("x is a string"))
        .expect("x is base64url");
    let y = URL_SAFE_NO_PAD
        .decode(proof_key["y"].as_str().expect("y is a string"))
        .expect("y is base64url");

    assert_eq!(x.len(), 32, "x is padded to 32 bytes");
    assert_eq!(y.len(), 32, "y is padded to 32 bytes");

    let mut sec1 = Vec::with_capacity(65);
    sec1.push(0x04);
    sec1.extend_from_slice(&x);
    sec1.extend_from_slice(&y);
    let key = VerifyingKey::from_sec1_bytes(&sec1).expect("the JWK is a valid P-256 key");

    let header = request
        .headers
        .get("Signature")
        .expect("the request has a Signature header")
        .to_str()
        .expect("the Signature header is ASCII");
    let block = STANDARD
        .decode(header)
        .expect("the Signature header is standard base64");
    assert_eq!(block.len(), 76);

    let filetime = FileTime::from_ticks(u64::from_be_bytes(
        block[4..12].try_into().expect("eight FILETIME bytes"),
    ));
    let signature = Signature::from_slice(&block[12..]).expect("64-byte r || s signature");
    let path = path_and_query(request.url.as_str()).expect("the request URL has a path");
    let payload = PayloadBuilder::new(filetime, "POST", &path, &request.body).build();

    key.verify(&payload, &signature)
        .expect("the signature covers the exact body");
}

#[test]
fn the_defaults_are_the_live_endpoints_and_five_seconds() {
    let auth = Auth::new();

    assert_eq!(auth.slow_down_increment, Duration::from_secs(5));
    assert_eq!(
        auth.endpoints.device_code,
        "https://login.live.com/oauth20_connect.srf"
    );
    assert_eq!(
        auth.endpoints.device_authenticate,
        "https://device.auth.xboxlive.com/device/authenticate"
    );
    assert_eq!(
        auth.endpoints.sisu_authorize,
        "https://sisu.xboxlive.com/authorize"
    );
}

#[tokio::test]
async fn device_code_request_returns_the_user_instructions() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;

    let code = test_auth(&server)
        .request_device_code()
        .await
        .expect("the device code is parsed");

    assert_eq!(code.user_code(), "USER-CODE");
    assert_eq!(code.verification_uri(), "https://example.test/device");
    assert_eq!(code.interval(), Duration::ZERO);
    assert_eq!(code.expires_in(), Duration::from_secs(900));

    let debug = format!("{code:?}");
    assert!(debug.contains("USER-CODE"));
    assert!(debug.contains("https://example.test/device"));
    assert!(!debug.contains("device-code-value"));

    let requests = recorded(&server, "/oauth20_connect.srf").await;
    assert_eq!(requests.len(), 1);
    let form = body_text(&requests[0]);
    assert!(form.contains("client_id=000000004424da1f"));
    assert!(form.contains("response_type=device_code"));
    assert!(form.contains("MBI_SSL"));
}

#[tokio::test]
async fn full_flow_signs_both_requests_and_builds_the_header() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_token(
        &server,
        Sequence::new(vec![
            json_response(400, json!({ "error": "authorization_pending" })),
            json_response(400, json!({ "error": "slow_down" })),
            token_success(),
        ]),
    )
    .await;
    mount_device_auth(&server, device_auth_success()).await;
    mount_sisu(&server, sisu_success()).await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    let live = auth.poll_device_code(&code).await.expect("token poll");

    let live_debug = format!("{live:?}");
    assert!(!live_debug.contains("msa-access-token"));
    assert!(!live_debug.contains("msa-refresh-token"));

    let xbox = auth.authenticate(&live).await.expect("signed auth flow");

    assert_eq!(xbox.uhs(), "1234567890");
    assert_eq!(
        xbox.authorization_header(),
        "XBL3.0 x=1234567890;xbox-token-value"
    );

    let xbox_debug = format!("{xbox:?}");
    assert!(!xbox_debug.contains("xbox-token-value"));
    assert!(xbox_debug.contains("1234567890"));

    let token_requests = recorded(&server, "/oauth20_token.srf").await;
    assert_eq!(token_requests.len(), 3, "pending and slow_down are retried");
    for request in &token_requests {
        let form = body_text(request);
        assert!(form.contains("device_code=device-code-value"));
        assert!(form.contains("grant_type=urn"));
    }

    let device_requests = recorded(&server, "/device/authenticate").await;
    assert_eq!(device_requests.len(), 1);
    let device_body: serde_json::Value =
        serde_json::from_slice(&device_requests[0].body).expect("device body is JSON");
    assert_eq!(device_body["Properties"]["AuthMethod"], "ProofOfPossession");
    assert_eq!(device_body["Properties"]["DeviceType"], "Win32");
    assert_eq!(device_body["Properties"]["Version"], "0.0.0");
    assert_eq!(device_body["RelyingParty"], "http://auth.xboxlive.com");
    assert_eq!(device_body["TokenType"], "JWT");
    let id = device_body["Properties"]["Id"]
        .as_str()
        .expect("Id is a string");
    assert!(id.starts_with('{') && id.ends_with('}'));
    assert_eq!(id.len(), 38);
    verify_signed_request(&device_requests[0], &device_body["Properties"]["ProofKey"]);

    let sisu_requests = recorded(&server, "/sisu/authorize").await;
    assert_eq!(sisu_requests.len(), 1);
    let sisu_body: serde_json::Value =
        serde_json::from_slice(&sisu_requests[0].body).expect("SISU body is JSON");
    assert_eq!(sisu_body["AccessToken"], "t=msa-access-token");
    assert_eq!(sisu_body["AppId"], "000000004424da1f");
    assert_eq!(sisu_body["DeviceToken"], "device-token-value");
    assert_eq!(sisu_body["Sandbox"], "RETAIL");
    assert_eq!(sisu_body["UseModernGamertag"], true);
    assert_eq!(sisu_body["SiteName"], "user.auth.xboxlive.com");
    assert_eq!(sisu_body["RelyingParty"], "http://xboxlive.com");
    assert_eq!(
        device_body["Properties"]["ProofKey"], sisu_body["ProofKey"],
        "one key per relying party signs both requests"
    );
    verify_signed_request(&sisu_requests[0], &sisu_body["ProofKey"]);
}

#[tokio::test]
async fn a_400_from_the_token_endpoint_is_an_api_error() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_token(
        &server,
        json_response(
            400,
            json!({ "error": "invalid_grant", "detail": "secret-body-value" }),
        ),
    )
    .await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    let error = auth
        .poll_device_code(&code)
        .await
        .expect_err("400 is an error");

    match &error {
        AuthError::Api { status, body } => {
            assert_eq!(*status, 400);
            assert!(body.contains("invalid_grant"));
            assert!(body.contains("secret-body-value"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }

    let debug = format!("{error:?}");
    assert!(!debug.contains("invalid_grant"));
    assert!(!debug.contains("secret-body-value"));
    assert!(!error.to_string().contains("secret-body-value"));
}

#[tokio::test]
async fn a_400_from_device_authenticate_is_an_api_error() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_token(&server, token_success()).await;
    mount_device_auth(
        &server,
        json_response(400, json!({ "description": "bad signature" })),
    )
    .await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    let live = auth.poll_device_code(&code).await.expect("token poll");
    let error = auth.authenticate(&live).await.expect_err("400 is an error");

    match error {
        AuthError::Api { status, body } => {
            assert_eq!(status, 400);
            assert!(body.contains("bad signature"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_401_from_sisu_is_an_api_error() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_token(&server, token_success()).await;
    mount_device_auth(&server, device_auth_success()).await;
    mount_sisu(
        &server,
        json_response(401, json!({ "description": "token expired" })),
    )
    .await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    let live = auth.poll_device_code(&code).await.expect("token poll");
    let error = auth.authenticate(&live).await.expect_err("401 is an error");

    match error {
        AuthError::Api { status, body } => {
            assert_eq!(status, 401);
            assert!(body.contains("token expired"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn an_expired_device_code_is_an_auth_error() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_token(
        &server,
        json_response(400, json!({ "error": "expired_token" })),
    )
    .await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    let error = auth
        .poll_device_code(&code)
        .await
        .expect_err("an expired code is an error");

    match error {
        AuthError::Auth(message) => assert!(message.contains("expired")),
        other => panic!("expected an auth error, got {other:?}"),
    }
}

#[tokio::test]
async fn refresh_uses_the_refresh_grant_and_hides_both_tokens() {
    let server = MockServer::start().await;
    mount_token(
        &server,
        json_response(
            200,
            json!({
                "access_token": "new-access-token",
                "refresh_token": "new-refresh-token",
                "expires_in": 3600
            }),
        ),
    )
    .await;

    let refreshed = test_auth(&server)
        .refresh("old-refresh-token")
        .await
        .expect("refresh succeeds");

    assert_eq!(refreshed.refresh_token(), Some("new-refresh-token"));
    assert_eq!(refreshed.expires_in(), Duration::from_secs(3600));

    let debug = format!("{refreshed:?}");
    assert!(!debug.contains("new-access-token"));
    assert!(!debug.contains("new-refresh-token"));

    let requests = recorded(&server, "/oauth20_token.srf").await;
    assert_eq!(requests.len(), 1);
    let form = body_text(&requests[0]);
    assert!(form.contains("grant_type=refresh_token"));
    assert!(form.contains("refresh_token=old-refresh-token"));
}

#[tokio::test]
async fn two_authenticate_calls_do_not_reuse_the_proof_key() {
    let server = MockServer::start().await;
    mount_device_code(&server).await;
    mount_device_auth(&server, device_auth_success()).await;
    mount_sisu(&server, sisu_success()).await;

    let auth = test_auth(&server);
    let code = auth.request_device_code().await.expect("device code");
    mount_token(&server, token_success()).await;

    for _ in 0..2 {
        let live = auth.poll_device_code(&code).await.expect("token poll");
        auth.authenticate(&live).await.expect("signed auth flow");
    }

    let device_requests = recorded(&server, "/device/authenticate").await;
    assert_eq!(device_requests.len(), 2);
    let first: serde_json::Value =
        serde_json::from_slice(&device_requests[0].body).expect("device body is JSON");
    let second: serde_json::Value =
        serde_json::from_slice(&device_requests[1].body).expect("device body is JSON");

    assert_ne!(
        first["Properties"]["ProofKey"]["x"],
        second["Properties"]["ProofKey"]["x"]
    );
}
