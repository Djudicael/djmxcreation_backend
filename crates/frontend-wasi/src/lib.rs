//! Admin and portfolio assets embedded at build time; no runtime filesystem.
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
    routing::get,
};

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

type Assets = &'static [(&'static str, &'static [u8])];

pub fn app_router(app: &str) -> Router {
    let assets = match app {
        "admin" => ADMIN,
        "portfolio" => PORTFOLIO,
        _ => panic!("FRONTEND_APP must be admin or portfolio"),
    };
    Router::new()
        .route("/", get(index))
        .route("/{*path}", get(serve))
        .with_state(assets)
}

async fn index(State(assets): State<Assets>) -> Response {
    serve_path(assets, "")
}

async fn serve(State(assets): State<Assets>, Path(path): Path<String>) -> Response {
    serve_path(assets, &path)
}

fn serve_path(assets: Assets, path: &str) -> Response {
    if path.split('/').any(|segment| matches!(segment, "." | ".."))
        || path == "api"
        || path.starts_with("api/")
    {
        return response(StatusCode::NOT_FOUND, "text/plain", b"not found");
    }
    let selected = assets.iter().find(|(name, _)| *name == path).or_else(|| {
        if path.is_empty() || !path.rsplit('/').next().unwrap_or("").contains('.') {
            assets.iter().find(|(name, _)| *name == "index.html")
        } else {
            None
        }
    });
    match selected {
        Some((name, bytes)) => response(StatusCode::OK, content_type(name), bytes),
        None => response(StatusCode::NOT_FOUND, "text/plain", b"not found"),
    }
}

fn response(status: StatusCode, mime: &'static str, bytes: &'static [u8]) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "no-store")
        .header("x-content-type-options", "nosniff")
        .body(Body::from(bytes))
        .unwrap()
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "json" => "application/json",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

#[cfg(target_arch = "wasm32")]
#[wstd_axum::http_server]
fn main() -> Router {
    app_router(&std::env::var("FRONTEND_APP").expect("FRONTEND_APP is required"))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn embeds_both_apps_and_does_not_return_html_for_missing_assets_or_api_routes() {
        for app in ["admin", "portfolio"] {
            let server = axum_test::TestServer::new(super::app_router(app));
            server.get("/").await.assert_status_ok();
            server.get("/projects/example").await.assert_status_ok();
            let js = server.get("/js/index.js").await;
            js.assert_status_ok();
            js.assert_header("content-type", "text/javascript; charset=utf-8");
            server
                .get("/ressource/missing.svg")
                .await
                .assert_status_not_found();
            server
                .get("/api/portfolio/v1/projects")
                .await
                .assert_status_not_found();
            server
                .post("/")
                .await
                .assert_status(axum::http::StatusCode::METHOD_NOT_ALLOWED);
            if app == "admin" {
                server
                    .get("/ressource/icon/pen%20tool.svg")
                    .await
                    .assert_status_ok();
            }
        }
    }
}
