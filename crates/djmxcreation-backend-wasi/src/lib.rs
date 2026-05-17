pub mod error;
pub mod router;
pub mod server;
pub mod service;
pub mod validation;

use axum::Router;

/// Build the router for tests (native) and for WASI runtime (wasm32).
pub fn app_router() -> Router {
    server::starter::build_router()
}

/// WASI P2 HTTP server entry point — only compiled when targeting wasm32.
#[cfg(target_arch = "wasm32")]
#[wstd_axum::http_server]
fn main() -> Router {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
    app_router()
}
