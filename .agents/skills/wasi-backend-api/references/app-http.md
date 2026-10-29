# `app-http`: runtime adapters

Every server in this repository makes outbound HTTP calls, waits on timers,
and applies timeouts through this one small crate. It exposes **the same
API on both targets**, so business code, repositories, and handlers need no
`cfg`:

| Function | WASI component (P3) | Native |
|---|---|---|
| `send(Request<Body>) -> Result<Response<Body>, SendError>` | `wasi:http` client | `reqwest` (rustls) |
| `read_body(Response<Body>, max_bytes) -> Result<Bytes, axum::Error>` | same | same |
| `sleep(Duration)` | P3 monotonic clock | `tokio::time::sleep` |
| `timeout(Duration, on_timeout, work)` | race with `futures-concurrency` | `tokio::time::timeout` |
| `handle(Router, wasi Request)` | P3 entry-point bridge | — (native uses `axum::serve`) |

The whole crate compiles for both `x86_64-unknown-linux-gnu` and
`wasm32-wasip2`; all futures are `Send` (the AWS SDK requires it).

## `crates/app-http/Cargo.toml`

```toml
[package]
name = "app-http"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[[example]]
name = "http-client-smoke"
crate-type = ["cdylib"]

[dependencies]
axum = { workspace = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
tower = { workspace = true, features = ["util"] }
wasip3 = { workspace = true }          # features = ["http-compat"] at workspace level
futures-concurrency = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { workspace = true, features = ["time"] }
reqwest = { workspace = true }
```

## `crates/app-http/src/lib.rs`

```rust
//! Runtime adapters: outbound HTTP, sleep, and timeouts with one API for
//! native (tokio + reqwest) and WASI 0.3 components (P3 executor).

use std::time::Duration;

pub use axum::body::{Body, Bytes};
pub use axum::http::{Request, Response};

/// An outbound request failed before a response arrived.
#[derive(Debug)]
pub struct SendError(pub String);

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "outbound request failed: {}", self.0)
    }
}

impl std::error::Error for SendError {}

/// Read a response body, failing when it exceeds `max_bytes`.
pub async fn read_body(response: Response<Body>, max_bytes: usize) -> Result<Bytes, axum::Error> {
    axum::body::to_bytes(response.into_body(), max_bytes).await
}

/// Run `work`, returning `Err(on_timeout)` if it does not finish in time.
pub async fn timeout<T, E>(
    duration: Duration,
    on_timeout: E,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    #[cfg(target_arch = "wasm32")]
    {
        use futures_concurrency::future::Race;
        let expired = async {
            sleep(duration).await;
            Err(on_timeout)
        };
        (work, expired).race().await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::timeout(duration, work)
            .await
            .unwrap_or(Err(on_timeout))
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::{Body, Request, Response, SendError};
    use axum::body::HttpBody;
    use std::time::Duration;
    use tower::ServiceExt;
    use wasip3::http::types::{self, ErrorCode};
    use wasip3::http_compat::{
        http_from_wasi_request, http_from_wasi_response, http_into_wasi_request,
        http_into_wasi_response,
    };

    /// Dispatch an incoming P3 request through an Axum router.
    pub async fn handle(
        router: axum::Router,
        request: types::Request,
    ) -> Result<types::Response, ErrorCode> {
        let request = http_from_wasi_request(request)?.map(Body::new);
        let response = router
            .oneshot(request)
            .await
            .unwrap_or_else(|never| match never {});
        http_into_wasi_response(response)
    }

    pub async fn send(mut request: Request<Body>) -> Result<Response<Body>, SendError> {
        // Keep a known body length: some upstreams (RustFS) reject chunked framing.
        if !request
            .headers()
            .contains_key(axum::http::header::CONTENT_LENGTH)
            && let Some(length) = request.body().size_hint().exact()
        {
            request.headers_mut().insert(
                axum::http::header::CONTENT_LENGTH,
                axum::http::HeaderValue::from(length),
            );
        }
        let error = |e: ErrorCode| SendError(format!("{e:?}"));
        let request = http_into_wasi_request(request).map_err(error)?;
        let response = wasip3::http::client::send(request).await.map_err(error)?;
        Ok(http_from_wasi_response(response).map_err(error)?.map(Body::new))
    }

    pub async fn sleep(duration: Duration) {
        let nanos = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        wasip3::clocks::monotonic_clock::wait_for(nanos).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::{Body, Request, Response, SendError};
    use std::sync::OnceLock;
    use std::time::Duration;

    fn client() -> &'static reqwest::Client {
        static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
        CLIENT.get_or_init(reqwest::Client::new)
    }

    pub async fn send(request: Request<Body>) -> Result<Response<Body>, SendError> {
        let error = |e: &dyn std::fmt::Display| SendError(e.to_string());
        let (parts, body) = request.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| error(&e))?;
        let request = reqwest::Request::try_from(Request::from_parts(parts, body))
            .map_err(|e| error(&e))?;
        let response = client().execute(request).await.map_err(|e| error(&e))?;
        Ok(Response::from(response).map(Body::new))
    }

    pub async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

pub use imp::*;
```

Callers always pass an explicit `max_bytes` to `read_body` and always wrap
calls in `timeout`. Use small caps for metadata documents (OIDC discovery
and JWKS: 256 KiB) and the configured media limits for object-storage
responses.

## Using it

```rust
let request = app_http::Request::get(url)
    .body(app_http::Body::empty())
    .map_err(|_| AuthError::Unavailable)?;
let body = app_http::timeout(Duration::from_secs(5), AuthError::Unavailable, async {
    let response = app_http::send(request).await.map_err(|_| AuthError::Unavailable)?;
    if !response.status().is_success() {
        return Err(AuthError::Unavailable);
    }
    app_http::read_body(response, MAX_DOCUMENT_BYTES)
        .await
        .map_err(|_| AuthError::Unavailable)
})
.await?;
```

This one function body works on both targets.

## The smoke example

`examples/http-client-smoke.rs` is a tiny P3 component that exercises the
adapters inside a real Wasmtime: it serves `GET /`, makes one outbound call
to a URL from an env var, opens a PostgreSQL connection with
`wasi-pg-client`, and checks that a `timeout` fires. `dev-server
runtime-test` builds and runs it (see `wasi-testing`). Keep it small; it
exists to prove that HTTP, sockets, and clocks share one executor.

## Do not

- Do not call `wasip3::http::client::send`, `wasip3::clocks`,
  `reqwest`, or `tokio::time` directly from other crates. Add a function
  here instead.
- Do not add `wstd` or the `wasi` 0.14 bindings to this crate.
- Do not buffer unbounded response bodies.
