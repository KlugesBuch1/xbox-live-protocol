//! Offline tests for the profile client against a wiremock server.

use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xbl::Error;
use xbl::auth::XboxToken;
use xbl::client::Client;

/// The documented response shape.
const FIXTURE: &str = include_str!("fixtures/profile-me.json");

fn client(server: &MockServer) -> Client {
    Client::with_base_url(
        XboxToken::new("xbox-token-value", "1234567890"),
        server.uri(),
    )
}

#[tokio::test]
async fn a_200_parses_the_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/me/profile/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE))
        .mount(&server)
        .await;

    let profile = client(&server).profile().await.expect("the fixture parses");

    assert_eq!(profile.id, "{xuid}");
    assert_eq!(profile.host_id, "{xuid}");
    assert_eq!(profile.gamertag(), Some("Contoso"));
    assert_eq!(profile.gamerscore(), Some("12345"));
    assert_eq!(profile.setting("Bio"), Some("Playing on PC."));
    assert_eq!(profile.setting("Unknown"), None);
    assert_eq!(profile.settings.len(), 8);
    assert!(!profile.is_sponsored_user);
}

#[tokio::test]
async fn the_request_sends_the_token_the_contract_and_the_settings() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/me/profile/settings"))
        .and(header(
            "authorization",
            "XBL3.0 x=1234567890;xbox-token-value",
        ))
        .and(header("x-xbl-contract-version", "2"))
        .and(query_param(
            "settings",
            "Gamertag,ModernGamertag,Gamerscore,GameDisplayPicRaw,AccountTier,Bio,Location,XboxOneRep",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE))
        .mount(&server)
        .await;

    client(&server)
        .profile()
        .await
        .expect("the mock matches the request");
}

#[tokio::test]
async fn a_401_is_an_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/me/profile/settings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "code": 401,
            "description": "The client must authenticate"
        })))
        .mount(&server)
        .await;

    let error = client(&server)
        .profile()
        .await
        .expect_err("401 is an error");

    match &error {
        Error::Api { status, body } => {
            assert_eq!(*status, 401);
            assert!(body.contains("must authenticate"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }

    let debug = format!("{error:?}");
    assert!(!debug.contains("must authenticate"), "the body is redacted");
    assert!(!error.to_string().contains("must authenticate"));
}

#[tokio::test]
async fn an_empty_profile_users_list_is_its_own_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/me/profile/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "profileUsers": [] })))
        .mount(&server)
        .await;

    let error = client(&server)
        .profile()
        .await
        .expect_err("an empty list is an error");

    assert!(
        matches!(error, Error::EmptyProfileUsers),
        "expected EmptyProfileUsers, got {error:?}"
    );
    assert!(error.to_string().contains("profileUsers"));
}
