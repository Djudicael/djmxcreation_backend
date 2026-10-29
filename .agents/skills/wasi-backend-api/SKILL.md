---
name: wasi-backend-api
description: Build or change the DJMXCreation API server (app-api, currently the djmxcreation-backend-wasi crate) and the app-http runtime-adapter crate. One axum router runs as a native tokio binary or as a WASI 0.3 (P3 async) component through the wasip3 HTTP handler. Use when adding routes, handlers, middleware, OIDC bearer-token authentication, configuration, error-to-HTTP mapping, health or metrics endpoints, outbound HTTP calls, sleeps or timeouts, the ServiceRegister composition root, or the native/P3 entry points of any server.
---

# API server (`app-api`)

Read `wasi-app-architecture` first for crate boundaries. This skill covers the
API crate, the HTTP side of the media worker, and the shared `app-http`
adapters. Read [the adapter reference](references/app-http.md) before you
create or change `app-http`, or before any outbound call, sleep, or timeout.

The crate is `crates/djmxcreation-backend-wasi` today; the target name is
`app-api` (see the refactor map in `wasi-app-architecture`).

## 1. Two entry points, one router (native binary and P3 component)

`src/lib.rs` builds the router in one place for both targets and exports the
P3 handler on wasm:

```rust
pub mod auth;
pub mod error;
pub mod middleware;
pub mod router;
pub mod server;
pub mod service;
pub mod validation;

use axum::Router;
use app_config::Config;

/// Router built from environment variables (WASI runtime and native binary).
pub fn app_router() -> Router {
    server::starter::build_router()
}

/// Router built from explicit configuration (tests).
pub fn app_router_with(config: Config) -> Router {
    server::starter::build_router_with(config)
}

#[cfg(target_arch = "wasm32")]
wasip3::http::service::export!(HttpServer);

#[cfg(target_arch = "wasm32")]
struct HttpServer;

#[cfg(target_arch = "wasm32")]
impl wasip3::exports::http::handler::Guest for HttpServer {
    async fn handle(
        request: wasip3::http::types::Request,
    ) -> Result<wasip3::http::types::Response, wasip3::http::types::ErrorCode> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init();
        app_http::handle(app_router(), request).await
    }
}
```

`src/main.rs` is the native development server:

```rust
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    app_api::server::starter::run_tcp(app_api::app_router()).await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("native main is not used on wasm32; the P3 entry point is in lib.rs");
}
```

The same pattern applies to `frontend-server` and `media-worker`:
an `HttpServer` struct whose `Guest::handle` calls
`app_http::handle(<crate>::app_router(..), request)`.

In a component, HTTP, clocks, and `wasi-pg-client` sockets all share the P3
async executor; natively everything runs on tokio. Do **not** use `wstd` /
`wstd-axum` (P2 executor) or block on futures.

Under WASI the host may instantiate the component **per request**; natively
one process serves many requests. Write for the stricter case: treat
`build_router()` as cheap and stateless, with no connection pools and no
background tasks. Caches (for example the JWKS cache) are best-effort only.

## 2. Composition root

`server/starter.rs` has three functions:

- `build_router()`: loads `Config::from_env()` and calls the next function.
  A missing required variable panics with the variable **name**, never its
  value.
- `build_router_with(config: Config) -> Router`: builds the database config,
  storage client, OIDC verifier, the `ServiceRegister`, and the routers, then
  applies layers. **Tests call this function** with explicit config instead of
  setting environment variables.
- `run_tcp(app)` (native only): binds `BIND_ADDRESS:PORT`, uses
  `into_make_service_with_connect_info::<SocketAddr>()`, and shuts down
  gracefully on Ctrl-C/SIGTERM.

`service/service_register.rs` wires concrete adapters into services once:

```rust
#[derive(Clone)]
pub struct ServiceRegister {
    pub project_service: DynIProjectService,
    pub about_me_service: DynIAboutMeService,
    pub contact_service: DynIContactService,
    pub home_service: DynIHomeService,
}

impl ServiceRegister {
    pub fn new(db: Arc<DatabaseConfig>, storage: StorageClient, bucket: String) -> Self {
        let project_repository = Arc::new(ProjectRepository::new(db.clone()));
        let storage_repository = Arc::new(StorageRepository::new(storage));
        Self {
            project_service: Arc::new(ProjectService::new(
                project_repository, storage_repository.clone(), bucket.clone(),
            )),
            // …
        }
    }
}
```

This is the only place that names concrete repository types.

## 3. Routers and handlers

One file per feature: `router/<feature>_router.rs`. Mount it with `nest`
under its API prefix:

```rust
Router::new()
    .nest("/api/home", HomeRouter::new_router(register.clone()))
    .nest("/api/about", AboutMeRouter::new_router(register.clone()))
    .nest("/api/portfolio", ProjectRouter::new_router(register.clone()))
    .nest("/api/contact", ContactRouter::new_router(register.clone()))
```

Inside a router, routes carry their version: `/v1/projects`,
`/v1/projects/{id}`, `/v2/projects`.

```rust
pub struct ProjectRouter;

impl ProjectRouter {
    pub fn new_router(register: ServiceRegister) -> Router {
        Router::new()
            .route("/v2/projects", get(Self::list))
            .route("/v1/projects/{id}", get(Self::find).put(Self::update))
            .with_state(register.project_service)
    }

    async fn find(
        State(service): State<DynIProjectService>,
        Path(id): Path<Uuid>,
    ) -> ApiResult<Json<ProjectView>> {
        Ok(Json(service.find_project(id).await?))
    }
}
```

Rules:

- Use axum 0.8 path syntax: `{id}` and `{*rest}`.
- Use `State<DynI…Service>` per router. `Extension(service)` is checked only
  at runtime; replace it when you touch a router that still uses it.
- Keep handlers thin. A handler extracts input, validates it at the
  boundary, calls one service method, and maps the result. Business rules
  belong in the service.
- Request bodies are dedicated `Deserialize` structs (`…Payload`) with
  `#[serde(rename_all = "camelCase")]`. Responses are `app_core::view` types.
- Validate path, query, and multipart inputs in `validation.rs`: pagination
  (`page ≥ 1`, `1 ≤ size ≤ 100`), file names (no `..`, `/`, `\`, or NUL),
  and length limits.
- Set explicit body limits: a global `DefaultBodyLimit::max(…)`, plus a
  per-route override for small uploads. Large media goes directly to storage
  through presigned URLs; see `wasi-object-storage`.

## 4. Errors to HTTP

`error.rs` in the API crate is the **only** place that knows HTTP status codes:

```rust
pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)] Domain(#[from] app_core::Error),
    #[error("{0}")]       BadRequest(String),
    #[error("forbidden")] Forbidden,
    #[error(transparent)] Multipart(#[from] axum::extract::multipart::MultipartError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "administrator role required".into()),
            ApiError::Domain(e) => {
                let status = StatusCode::from_u16(e.http_status())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                if status.is_server_error() {
                    tracing::error!(error = ?e, "request failed");
                    (status, "internal server error".into())
                } else {
                    (status, e.to_string())
                }
            }
            ApiError::Multipart(_) => (StatusCode::BAD_REQUEST, "invalid upload".into()),
        };
        (status, Json(json!({ "errors": { "message": [message] } }))).into_response()
    }
}
```

The response shape is `{"errors": {"message": ["…"]}}`. The frontends parse
it; do not introduce a second shape.

## 5. Middleware stack

Apply layers in `build_router_with`, from the inside out:

1. `route_layer(from_fn_with_state(verifier, auth::authenticate))` on the
   API routers. Public routes (`/ping`, `/health*`, the internal maintenance
   route with its own token) are merged **after** this layer.
2. `DefaultBodyLimit`.
3. `TraceLayer::new_for_http()`.
4. CORS from `CORS_ORIGINS` (comma-separated). An empty list means no
   cross-origin access. Never use `Any` origins on authenticated APIs.
5. Security headers: `x-content-type-options: nosniff`, `referrer-policy:
   no-referrer`, `x-frame-options: DENY`.
6. Native only (`#[cfg(not(target_arch = "wasm32"))]`): request timeout
   (`tower::timeout` + `HandleErrorLayer`) and Prometheus `/metrics`. On
   wasm32 the `/metrics` route is omitted.

## 6. Health and observability

- `GET /ping` → `{"message":"API is responsive"}` (liveness, no I/O).
- `GET /health/ready` → opens a DB connection and runs `SELECT 1` within a
  2-second `app_http::timeout`. Returns 503 on failure.
- Log with structured fields: `tracing::info!(project_id = %id, "created")`.
  Never log tokens, passwords, presigned URLs, or connection strings.

## 7. Authentication (OIDC resource server)

The API accepts bearer tokens from an OpenID Connect provider (the OpenID
Connect WASI Hub in development). `auth.rs` holds the whole flow:

- `OidcConfig` comes from `app_config` (`OIDC_ISSUER`, `OIDC_AUDIENCE`,
  `OIDC_ADMIN_CLIENT_ID`, `OIDC_ADMIN_ROLE`). Require HTTPS for the issuer
  except on loopback.
- Discovery (`<issuer>/.well-known/openid-configuration`) → `jwks_uri`
  (must share the issuer origin) → `JwkSet`. Cache it for 5 minutes and
  refetch on an unknown `kid` at most every 30 seconds. Cap each document at
  256 KiB and each fetch at 5 seconds.
- Accept only `RS256` and `EdDSA`, and check that the JWK type matches the
  header algorithm. Validate `iss`, `aud`, and `exp`.
- A caller is an admin when `OIDC_ADMIN_ROLE` appears in
  `realm_access.roles` or in `resource_access.<OIDC_ADMIN_CLIENT_ID>.roles`.
  On success, insert
  `Extension<AdminIdentity>` into the request. Public `GET` routes stay
  anonymous; handlers that behave differently for admins take
  `Option<Extension<AdminIdentity>>`.
- Return 401 without detail for bad tokens, 403 for a missing role, and 503
  when keys are unavailable.

## 8. Outbound HTTP

One function, no `cfg`: `app-http` picks `wasi:http` or `reqwest` per
target. Always set a timeout and a size cap:

```rust
async fn get_document(&self, url: &str) -> Result<Vec<u8>, AuthError> {
    let request = app_http::Request::get(url)
        .body(app_http::Body::empty())
        .map_err(|_| AuthError::Unavailable)?;
    app_http::timeout(Duration::from_secs(5), AuthError::Unavailable, async {
        let response = app_http::send(request).await.map_err(|_| AuthError::Unavailable)?;
        if !response.status().is_success() {
            return Err(AuthError::Unavailable);
        }
        let body = app_http::read_body(response, MAX_DOCUMENT_BYTES)
            .await
            .map_err(|_| AuthError::Unavailable)?;
        Ok(body.to_vec())
    })
    .await
}
```

Validate outbound URLs before calling them (HTTPS outside local
development). TLS is handled by the host (WASI) or rustls (native).

## 9. Configuration variables

Standard names (extend them, but do not rename them):

| Variable | Meaning |
|---|---|
| `DATABASE_URL` | `postgresql://user:pass@host:5432/db` |
| `STORAGE_ENDPOINT`, `STORAGE_ACCESS_KEY`, `STORAGE_SECRET_KEY`, `STORAGE_REGION`, `STORAGE_BUCKET` | S3/RustFS |
| `OIDC_ISSUER`, `OIDC_AUDIENCE`, `OIDC_ADMIN_CLIENT_ID`, `OIDC_ADMIN_ROLE` | auth (required; authentication is never disabled) |
| `CORS_ORIGINS` | comma-separated browser origins |
| `MEDIA_OPTIMIZATION_ENABLED`, `MEDIA_WORKER_URL`, `MEDIA_PROCESSING_TOKEN` | media pipeline |
| `FRONTEND_APP` | `admin` or `portfolio` (frontend server) |
| `PORT`, `BIND_ADDRESS` | native binaries (the WASI host owns the listener) |
| `RUST_LOG` | tracing filter |

Every new variable goes into `app_config`, `.env.template`, and the
`deploy/wcp/*.example.toml` files that need it.

## Done when

- `cargo build -p app-api` and
  `cargo build --target wasm32-wasip2 -p app-api --lib` both pass, and
  `cargo tree -i wstd --target wasm32-wasip2` finds nothing.
- No new `cfg(target_arch …)` outside the entry points and `app-http`.
- A router test exercises the new route through `app_router_with(config)`
  (see `wasi-testing`).
- Error paths return the documented shape and log server-side details.
