//! Resource-server authentication for OpenID Connect WASI Hub realm tokens.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use serde::Deserialize;
use url::Url;

const CACHE_TTL: Duration = Duration::from_secs(300);
const REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const MAX_DOCUMENT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub struct OidcConfig {
    issuer: String,
    audience: String,
    admin_client_id: String,
    admin_role: String,
}

impl OidcConfig {
    pub fn new(
        issuer: &str,
        audience: &str,
        admin_client_id: &str,
        admin_role: &str,
    ) -> anyhow::Result<Self> {
        let issuer = issuer.trim_end_matches('/');
        validate_url(issuer)?;
        anyhow::ensure!(
            !audience.trim().is_empty(),
            "OIDC_AUDIENCE must not be empty"
        );
        anyhow::ensure!(
            !admin_client_id.trim().is_empty(),
            "OIDC_ADMIN_CLIENT_ID must not be empty"
        );
        anyhow::ensure!(
            !admin_role.trim().is_empty(),
            "OIDC_ADMIN_ROLE must not be empty"
        );
        Ok(Self {
            issuer: issuer.into(),
            audience: audience.into(),
            admin_client_id: admin_client_id.into(),
            admin_role: admin_role.into(),
        })
    }

    pub fn from_env() -> anyhow::Result<Self> {
        Self::new(
            &std::env::var("OIDC_ISSUER")
                .map_err(|_| anyhow::anyhow!("OIDC_ISSUER is required"))?,
            &std::env::var("OIDC_AUDIENCE")
                .map_err(|_| anyhow::anyhow!("OIDC_AUDIENCE is required"))?,
            &std::env::var("OIDC_ADMIN_CLIENT_ID").unwrap_or_else(|_| "portfolio".into()),
            &std::env::var("OIDC_ADMIN_ROLE").unwrap_or_else(|_| "admin".into()),
        )
    }
}

fn validate_url(value: &str) -> anyhow::Result<Url> {
    let url = Url::parse(value)?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    anyhow::ensure!(
        url.scheme() == "https" || (url.scheme() == "http" && loopback),
        "OIDC URLs require HTTPS (HTTP is allowed on loopback for development)"
    );
    anyhow::ensure!(
        url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "invalid OIDC URL"
    );
    Ok(url)
}

#[derive(Default)]
struct KeyCache {
    keys: Option<JwkSet>,
    fetched_at: Option<Instant>,
    attempted_at: Option<Instant>,
}

#[derive(Clone)]
pub struct OidcVerifier {
    config: OidcConfig,
    cache: Arc<Mutex<KeyCache>>,
    refresh: Arc<futures::lock::Mutex<()>>,
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct AdminIdentity {
    pub subject: String,
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[derive(Deserialize, Default)]
struct Roles {
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Deserialize)]
struct AccessClaims {
    sub: String,
    iat: u64,
    // Hub ID tokens have no scope. Requiring it rejects their use as API tokens.
    scope: String,
    #[serde(default)]
    cnf: Option<serde_json::Value>,
    #[serde(default)]
    realm_access: Roles,
    #[serde(default)]
    resource_access: HashMap<String, Roles>,
}

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    Forbidden,
    Unavailable,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "invalid or missing access token"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "administrator role required"),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "identity provider unavailable",
            ),
        };
        let mut response = (status, Json(serde_json::json!({"error": message}))).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, "Bearer".parse().unwrap());
        }
        response
    }
}

impl OidcVerifier {
    pub fn new(config: OidcConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            cache: Arc::new(Mutex::new(KeyCache::default())),
            refresh: Arc::new(futures::lock::Mutex::new(())),
            #[cfg(not(target_arch = "wasm32"))]
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }

    async fn fetch_keys(&self) -> anyhow::Result<JwkSet> {
        let discovery: Discovery = serde_json::from_slice(
            &self
                .get_document(&format!(
                    "{}/.well-known/openid-configuration",
                    self.config.issuer
                ))
                .await?,
        )?;
        anyhow::ensure!(
            discovery.issuer == self.config.issuer,
            "discovery issuer mismatch"
        );
        let jwks_url = validate_url(&discovery.jwks_uri)?;
        let issuer_url = validate_url(&self.config.issuer)?;
        anyhow::ensure!(
            jwks_url.origin() == issuer_url.origin(),
            "JWKS must use the issuer origin"
        );
        let keys: JwkSet = serde_json::from_slice(&self.get_document(&discovery.jwks_uri).await?)?;
        anyhow::ensure!(!keys.keys.is_empty(), "empty JWKS");
        Ok(keys)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn get_document(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let mut response = self.client.get(url).send().await?;
        anyhow::ensure!(response.status().is_success(), "OIDC endpoint failed");
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            anyhow::ensure!(
                bytes.len() + chunk.len() <= MAX_DOCUMENT_BYTES,
                "OIDC document too large"
            );
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_document(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        use wstd::http::{
            Client, Request,
            body::{Body, util::BodyExt},
        };
        let mut client = Client::new();
        client.set_connect_timeout(Duration::from_secs(5));
        client.set_first_byte_timeout(Duration::from_secs(5));
        client.set_between_bytes_timeout(Duration::from_secs(5));
        let request = Request::get(url).body(Body::empty())?;
        let response = client.send(request).await?;
        anyhow::ensure!(response.status().is_success(), "OIDC endpoint failed");
        let mut body = response.into_body().into_boxed_body();
        let mut bytes = Vec::new();
        while let Some(frame) = body.frame().await {
            if let Ok(chunk) = frame?.into_data() {
                anyhow::ensure!(
                    bytes.len() + chunk.len() <= MAX_DOCUMENT_BYTES,
                    "OIDC document too large"
                );
                bytes.extend_from_slice(&chunk);
            }
        }
        Ok(bytes)
    }

    pub async fn verify(&self, token: &str) -> Result<Option<AdminIdentity>, AuthError> {
        if token.len() > 16 * 1024 {
            return Err(AuthError::Unauthorized);
        }
        let header = decode_header(token).map_err(|_| AuthError::Unauthorized)?;
        if !matches!(header.alg, Algorithm::RS256 | Algorithm::EdDSA) {
            return Err(AuthError::Unauthorized);
        }
        let kid = header
            .kid
            .as_deref()
            .filter(|kid| !kid.is_empty())
            .ok_or(AuthError::Unauthorized)?;
        let refresh = self.refresh.lock().await;
        let should_fetch = {
            let mut cache = self.cache.lock().map_err(|_| AuthError::Unavailable)?;
            let fresh = cache.fetched_at.is_some_and(|at| at.elapsed() < CACHE_TTL);
            let known = cache
                .keys
                .as_ref()
                .is_some_and(|keys| keys.find(kid).is_some());
            if fresh && known {
                false
            } else if cache
                .attempted_at
                .is_some_and(|at| at.elapsed() < REFRESH_INTERVAL)
            {
                return Err(if fresh {
                    AuthError::Unauthorized
                } else {
                    AuthError::Unavailable
                });
            } else {
                cache.attempted_at = Some(Instant::now());
                true
            }
        };
        if should_fetch {
            let keys = self.fetch_keys().await.map_err(|error| {
                tracing::warn!(%error, "OIDC signing-key refresh failed");
                AuthError::Unavailable
            })?;
            let mut cache = self.cache.lock().map_err(|_| AuthError::Unavailable)?;
            cache.keys = Some(keys);
            cache.fetched_at = Some(Instant::now());
        }
        drop(refresh);
        let key = {
            let cache = self.cache.lock().map_err(|_| AuthError::Unavailable)?;
            let jwk = cache
                .keys
                .as_ref()
                .and_then(|keys| keys.find(kid))
                .ok_or(AuthError::Unauthorized)?;
            let expected_alg = if header.alg == Algorithm::RS256 {
                jsonwebtoken::jwk::KeyAlgorithm::RS256
            } else {
                jsonwebtoken::jwk::KeyAlgorithm::EdDSA
            };
            match (&jwk.algorithm, header.alg) {
                (jsonwebtoken::jwk::AlgorithmParameters::RSA(_), Algorithm::RS256) => {}
                (
                    jsonwebtoken::jwk::AlgorithmParameters::OctetKeyPair(parameters),
                    Algorithm::EdDSA,
                ) if parameters.curve == jsonwebtoken::jwk::EllipticCurve::Ed25519 => {}
                _ => return Err(AuthError::Unauthorized),
            }
            if jwk
                .common
                .key_algorithm
                .is_some_and(|alg| alg != expected_alg)
                || jwk
                    .common
                    .public_key_use
                    .as_ref()
                    .is_some_and(|usage| *usage != jsonwebtoken::jwk::PublicKeyUse::Signature)
                || jwk
                    .common
                    .key_operations
                    .as_ref()
                    .is_some_and(|ops| !ops.contains(&jsonwebtoken::jwk::KeyOperations::Verify))
            {
                return Err(AuthError::Unauthorized);
            }
            let key = DecodingKey::from_jwk(jwk).map_err(|_| AuthError::Unauthorized)?;
            if header.alg == Algorithm::EdDSA
                && key
                    .try_get_as_bytes()
                    .map_err(|_| AuthError::Unauthorized)?
                    .len()
                    != 32
            {
                return Err(AuthError::Unauthorized);
            }
            key
        };
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub", "iat"]);
        validation.validate_nbf = true;
        validation.leeway = 30;
        let claims = decode::<AccessClaims>(token, &key, &validation)
            .map_err(|_| AuthError::Unauthorized)?
            .claims;
        if claims.sub.is_empty()
            || claims.scope.is_empty()
            || claims.iat > jsonwebtoken::get_current_timestamp() + 30
            || claims.cnf.is_some()
        {
            return Err(AuthError::Unauthorized);
        }
        let admin = claims.realm_access.roles.contains(&self.config.admin_role)
            || claims
                .resource_access
                .get(&self.config.admin_client_id)
                .is_some_and(|roles| roles.roles.contains(&self.config.admin_role));
        Ok(admin.then_some(AdminIdentity {
            subject: claims.sub,
        }))
    }
}

/// Authentication runs on every API route; public reads may omit a bearer token.
pub async fn authenticate(
    State(verifier): State<OidcVerifier>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let values = request.headers().get_all(header::AUTHORIZATION);
    let mut values = values.iter();
    let identity = if let Some(value) = values.next() {
        if values.next().is_some() {
            return Err(AuthError::Unauthorized);
        }
        let value = value.to_str().map_err(|_| AuthError::Unauthorized)?;
        let (scheme, token) = value.split_once(' ').ok_or(AuthError::Unauthorized)?;
        if !scheme.eq_ignore_ascii_case("Bearer")
            || token.is_empty()
            || token.contains(char::is_whitespace)
        {
            return Err(AuthError::Unauthorized);
        }
        verifier.verify(token).await?
    } else {
        None
    };
    let admin_required = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    ) || request.uri().path().trim_end_matches('/')
        == "/api/portfolio/v1/projects";
    if admin_required && identity.is_none() {
        return Err(if request.headers().contains_key(header::AUTHORIZATION) {
            AuthError::Forbidden
        } else {
            AuthError::Unauthorized
        });
    }
    if let Some(identity) = identity {
        request.extensions_mut().insert(identity);
    }
    Ok(next.run(request).await)
}
