mod support;

use axum::{Router, http::StatusCode, middleware, routing::get};
use axum_test::TestServer;
use djmxcreation_backend_wasi::auth::{AuthError, OidcConfig, authenticate};
use serde_json::json;
use std::sync::atomic::Ordering;
use support::{Hub, sign};

#[tokio::test]
async fn hub_discovery_validates_access_token_and_caches_keys() {
    let hub = Hub::start().await;
    let verifier = hub.verifier();
    for _ in 0..2 {
        assert_eq!(
            verifier
                .verify(&hub.token())
                .await
                .unwrap()
                .unwrap()
                .subject,
            "test-admin"
        );
    }
    assert_eq!(hub.requests.load(Ordering::SeqCst), 1);
    // Unknown keys do not trigger an HTTP request for every attacker-supplied kid.
    assert!(matches!(
        verifier.verify(&sign(&hub.claims(), "unknown")).await,
        Err(AuthError::Unauthorized)
    ));
    assert_eq!(hub.requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejects_wrong_issuer_audience_time_id_tokens_and_sender_bound_tokens() {
    let hub = Hub::start().await;
    let verifier = hub.verifier();
    let now = jsonwebtoken::get_current_timestamp();
    for (key, value) in [
        ("iss", json!("https://other.example/realms/portfolio")),
        ("aud", json!("another-api")),
        ("exp", json!(now - 120)),
        ("nbf", json!(now + 120)),
        ("iat", json!(now + 120)),
        ("sub", json!("")),
        ("cnf", json!({"jkt":"proof-required"})),
    ] {
        let mut claims = hub.claims();
        claims[key] = value;
        assert!(
            matches!(
                verifier.verify(&sign(&claims, "hub-ed")).await,
                Err(AuthError::Unauthorized)
            ),
            "accepted invalid {key}"
        );
    }
    for key in ["scope", "exp", "iss", "aud", "sub", "iat"] {
        let mut claims = hub.claims();
        claims.as_object_mut().unwrap().remove(key);
        assert!(
            matches!(
                verifier.verify(&sign(&claims, "hub-ed")).await,
                Err(AuthError::Unauthorized)
            ),
            "accepted missing {key}"
        );
    }
    let mut token = hub.token().into_bytes();
    let index = token.iter().position(|b| *b == b'.').unwrap() + 2;
    token[index] = if token[index] == b'A' { b'B' } else { b'A' };
    assert!(matches!(
        verifier.verify(&String::from_utf8(token).unwrap()).await,
        Err(AuthError::Unauthorized)
    ));
}

#[tokio::test]
async fn grants_only_configured_realm_or_client_admin_role() {
    let hub = Hub::start().await;
    let verifier = hub.verifier();
    let mut claims = hub.claims();
    claims["realm_access"] = json!({"roles":["reader"]});
    claims["resource_access"] = json!({"other-client":{"roles":["admin"]}});
    assert!(
        verifier
            .verify(&sign(&claims, "hub-ed"))
            .await
            .unwrap()
            .is_none()
    );
    claims["resource_access"]["portfolio"] = json!({"roles":["admin"]});
    assert!(
        verifier
            .verify(&sign(&claims, "hub-ed"))
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn middleware_protects_mutations_and_admin_reads_without_blocking_public_reads() {
    let hub = Hub::start().await;
    let router = Router::new()
        .route("/api/about/v1/me", get(|| async { "public" }))
        .route(
            "/api/portfolio/v1/projects",
            get(|| async { "admin" }).post(|| async { "created" }),
        )
        .route_layer(middleware::from_fn_with_state(hub.verifier(), authenticate));
    let server = TestServer::new(router);
    server.get("/api/about/v1/me").await.assert_status_ok();
    server
        .get("/api/portfolio/v1/projects")
        .await
        .assert_status_unauthorized();
    server
        .post("/api/portfolio/v1/projects")
        .await
        .assert_status_unauthorized();
    server
        .post("/api/portfolio/v1/projects")
        .add_header("Authorization", format!("Bearer {}", hub.token()))
        .await
        .assert_status_ok();
    let mut claims = hub.claims();
    claims["realm_access"] = json!({"roles":["reader"]});
    server
        .post("/api/portfolio/v1/projects")
        .add_header(
            "Authorization",
            format!("Bearer {}", sign(&claims, "hub-ed")),
        )
        .await
        .assert_status(StatusCode::FORBIDDEN);
    for token in ["Basic abc", "Bearer", "Bearer invalid", "Bearer a b"] {
        let response = server
            .post("/api/portfolio/v1/projects")
            .add_header("Authorization", token)
            .await;
        response.assert_status_unauthorized();
        response.assert_header("www-authenticate", "Bearer");
    }
}

#[tokio::test]
async fn rejects_jwks_keys_not_intended_for_signature_verification() {
    let hub = Hub::start().await;
    hub.keys.write().unwrap()["keys"][0]["use"] = json!("enc");
    assert!(matches!(
        hub.verifier().verify(&hub.token()).await,
        Err(AuthError::Unauthorized)
    ));
}

#[tokio::test]
async fn accepts_hub_rs256_tokens_and_rejects_symmetric_algorithm_confusion() {
    let hub = Hub::start().await;
    *hub.keys.write().unwrap() =
        serde_json::from_str(include_str!("support/test-rsa-jwks.json")).unwrap();
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("hub-rsa".into());
    // Generated test-only RSA key; never used by the application or real Hub.
    let token = jsonwebtoken::encode(
        &header,
        &hub.claims(),
        &jsonwebtoken::EncodingKey::from_rsa_der(include_bytes!("support/test-rsa.der")),
    )
    .unwrap();
    assert!(hub.verifier().verify(&token).await.unwrap().is_some());
    header.alg = jsonwebtoken::Algorithm::HS256;
    let token = jsonwebtoken::encode(
        &header,
        &hub.claims(),
        &jsonwebtoken::EncodingKey::from_secret(b"not-an-asymmetric-key"),
    )
    .unwrap();
    assert!(matches!(
        hub.verifier().verify(&token).await,
        Err(AuthError::Unauthorized)
    ));
}

#[tokio::test]
async fn rejects_malformed_ed25519_public_key_without_panicking() {
    let hub = Hub::start().await;
    hub.keys.write().unwrap()["keys"][0]["x"] = json!("AA");
    assert!(matches!(
        hub.verifier().verify(&hub.token()).await,
        Err(AuthError::Unauthorized)
    ));
}

#[tokio::test]
async fn provider_outage_fails_closed() {
    let hub = Hub::start().await;
    hub.keys.write().unwrap()["keys"] = json!([]);
    assert!(matches!(
        hub.verifier().verify(&hub.token()).await,
        Err(AuthError::Unavailable)
    ));
}

#[test]
fn rejects_unsafe_or_incomplete_configuration() {
    for issuer in [
        "http://identity.example/realms/portfolio",
        "https://user:secret@identity.example/realms/p",
        "https://identity.example/realms/p?override=1",
    ] {
        assert!(OidcConfig::new(issuer, "portfolio-api", "portfolio", "admin").is_err());
    }
    assert!(
        OidcConfig::new(
            "https://identity.example/realms/p",
            "",
            "portfolio",
            "admin"
        )
        .is_err()
    );
}
