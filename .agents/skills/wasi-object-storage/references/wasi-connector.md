# P3 `WasiHttpClient` connector and `WasiSleep`

On wasm, `aws-sdk-s3` runs with `default-features = false`: it has no HTTP
engine and no TLS. These two small types plug the host's `wasi:http` (through
`app-http`) and the P3 monotonic clock into the SDK. Their futures are
`Send`, as the SDK requires.

## `repository/src/config/wasi_http_client.rs`

```rust
//! AWS SDK HTTP connector that sends requests through the host's P3 `wasi:http`.
#![cfg(target_arch = "wasm32")]

use aws_sdk_s3::primitives::ByteStream;
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::http::{Response as SmithyResponse, StatusCode};
use aws_smithy_types::body::SdkBody;
use app_http::{Body, Request};

/// Largest response body the connector buffers (object reads included).
/// Keep it at or above the largest object a component downloads.
const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct WasiHttpClient;

impl WasiHttpClient {
    pub fn new() -> Self {
        Self
    }
}

fn io_error(context: &str, error: impl std::fmt::Debug) -> ConnectorError {
    ConnectorError::io(format!("{context}: {error:?}").into())
}

impl HttpConnector for WasiHttpClient {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        HttpConnectorFuture::new(async move {
            let method = request.method().to_string();
            let uri = request.uri().to_string();
            let headers = request.headers().clone();
            let body = ByteStream::new(request.into_body())
                .collect()
                .await
                .map_err(|e| io_error("read request body", e))?
                .into_bytes();

            let mut builder = Request::builder().method(method.as_str()).uri(uri.as_str());
            for (name, value) in headers.iter() {
                builder = builder.header(name, value);
            }
            // RustFS answers 411 to empty mutating calls without an explicit length.
            if body.is_empty()
                && matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
                && headers.get("content-length").is_none()
            {
                builder = builder.header("content-length", "0");
            }
            let request = builder
                .body(Body::from(body))
                .map_err(|e| ConnectorError::user(e.into()))?;

            let response = app_http::send(request)
                .await
                .map_err(|e| io_error("wasi:http send", e))?;
            let status = StatusCode::try_from(response.status().as_u16())
                .map_err(|e| io_error("status", e))?;
            let response_headers = response.headers().clone();
            let bytes = app_http::read_body(response, MAX_RESPONSE_BYTES)
                .await
                .map_err(|e| io_error("read response body", e))?;

            let mut out = SmithyResponse::new(status, SdkBody::from(bytes));
            for (name, value) in response_headers.iter() {
                if let Ok(value) = value.to_str() {
                    out.headers_mut().insert(name.as_str().to_owned(), value.to_owned());
                }
            }
            Ok(out)
        })
    }
}

impl HttpClient for WasiHttpClient {
    fn http_connector(
        &self,
        _settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        SharedHttpConnector::new(self.clone())
    }
}
```

Notes:

- `app_http::send` streams the body to the host and awaits the
  response on the shared P3 executor. There is no manual polling and no
  4096-byte write loop (those were P2 workarounds).
- `send` already sets `content-length` when the body size is known; the
  explicit `0` covers empty bodies whose size hint is not exact.
- Request bodies are collected in memory. That is fine for the small objects
  the components upload (resized variants, small images). Large uploads never
  pass through a component (see the direct-upload flow in `SKILL.md`).

## `WasiSleep` (in `config/storage.rs`)

```rust
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
struct WasiSleep;

#[cfg(target_arch = "wasm32")]
impl aws_sdk_s3::config::AsyncSleep for WasiSleep {
    fn sleep(&self, duration: std::time::Duration) -> aws_sdk_s3::config::Sleep {
        aws_sdk_s3::config::Sleep::new(app_http::sleep(duration))
    }
}
```

The SDK uses it for retries and timeouts. Without it, retry back-off panics
on wasm.

## Verifying the connector

- `cargo build --target wasm32-wasip2 -p app-api -p media-worker --lib`
  proves that the futures satisfy the SDK bounds.
- `dev-server smoke` in WASI mode exercises real uploads, presigned reads, and
  deletes against RustFS through the connector.
