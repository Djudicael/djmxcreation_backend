use std::sync::{
    Arc, RwLock,
    atomic::{AtomicUsize, Ordering},
};

use axum::{Json, Router, routing::get};
use djmxcreation_backend_wasi::auth::{OidcConfig, OidcVerifier};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};

// RFC 8032 test vector 1. This key is only used by the mock identity provider.
pub const PUBLIC_KEY: &str = "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo";

pub struct Hub {
    pub issuer: String,
    pub keys: Arc<RwLock<Value>>,
    pub requests: Arc<AtomicUsize>,
}

impl Hub {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}/realms/portfolio", listener.local_addr().unwrap());
        let discovery = json!({"issuer": issuer, "jwks_uri": format!("{issuer}/protocol/openid-connect/certs")});
        let keys = Arc::new(RwLock::new(
            json!({"keys": [{"kty":"OKP", "crv":"Ed25519", "kid":"hub-ed", "alg":"EdDSA", "use":"sig", "x":PUBLIC_KEY}]}),
        ));
        let requests = Arc::new(AtomicUsize::new(0));
        let route_keys = keys.clone();
        let route_requests = requests.clone();
        let router = Router::new()
            .route(
                "/realms/portfolio/.well-known/openid-configuration",
                get(move || {
                    let doc = discovery.clone();
                    async move { Json(doc) }
                }),
            )
            .route(
                "/realms/portfolio/protocol/openid-connect/certs",
                get(move || {
                    let keys = route_keys.clone();
                    let requests = route_requests.clone();
                    async move {
                        requests.fetch_add(1, Ordering::SeqCst);
                        Json(keys.read().unwrap().clone())
                    }
                }),
            );
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            issuer,
            keys,
            requests,
        }
    }

    pub fn verifier(&self) -> OidcVerifier {
        OidcVerifier::new(
            OidcConfig::new(&self.issuer, "portfolio-api", "portfolio", "admin").unwrap(),
        )
        .unwrap()
    }

    pub fn claims(&self) -> Value {
        let now = jsonwebtoken::get_current_timestamp();
        json!({"iss":self.issuer, "aud":["portfolio", "portfolio-api"], "sub":"test-admin", "iat":now, "exp":now+300, "scope":"openid profile roles", "realm_access":{"roles":["admin"]}})
    }

    pub fn token(&self) -> String {
        sign(&self.claims(), "hub-ed")
    }
}

pub fn sign(claims: &Value, kid: &str) -> String {
    let mut der = vec![
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20,
    ];
    let seed = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    der.extend(
        (0..seed.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&seed[i..i + 2], 16).unwrap()),
    );
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(kid.into());
    encode(&header, claims, &EncodingKey::from_ed_der(&der)).unwrap()
}
