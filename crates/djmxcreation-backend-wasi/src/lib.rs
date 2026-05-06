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

/// WASI p2 HTTP server entry point — only compiled when targeting wasm32.
#[cfg(target_arch = "wasm32")]
#[wstd_axum::http_server]
fn main() -> Router {
    app_router()
}
