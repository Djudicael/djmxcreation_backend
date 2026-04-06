use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::{Router, extract::OriginalUri, response::IntoResponse};
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .init();

    let target = env::args().nth(1).unwrap_or_else(|| "admin".to_string());
    let public_dir = format!("front/{target}");

    async fn spa_handler(path: String, dir: String) -> impl IntoResponse {
        // If the path looks like a static asset, return 404 (do not serve index.html)
        let is_asset = path.ends_with(".js")
            || path.ends_with(".css")
            || path.ends_with(".map")
            || path.ends_with(".json")
            || path.ends_with(".png")
            || path.ends_with(".jpg")
            || path.ends_with(".jpeg")
            || path.ends_with(".gif")
            || path.ends_with(".svg")
            || path.ends_with(".ico")
            || path.ends_with(".webp");
        if is_asset {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap_or_else(|_| Response::new(Body::from("Not found")));
        }
        // Always serve index.html for fallback
        let index_path = format!("{}/index.html", dir);
        match tokio::fs::read(&index_path).await {
            Ok(content) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Body::from(content))
                .unwrap_or_else(|_| Response::new(Body::from("index.html"))),
            Err(e) => {
                tracing::warn!(path = %index_path, error = %e, "failed to read index.html");
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("index.html non trouvé"))
                    .unwrap_or_else(|_| Response::new(Body::from("index.html non trouvé")))
            }
        }
    }

    let static_service = ServeDir::new(public_dir.clone());

    let app = Router::new()
        .route_service("/static/{*path}", static_service)
        .fallback({
            let public_dir = public_dir.clone();
            move |uri: OriginalUri| async move {
                let path = uri.0.path().to_string();
                let public_dir = public_dir.clone();
                spa_handler(path, public_dir).await
            }
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!(%addr, "SPA server running");

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            if let Err(error) = axum::serve(listener, app).await {
                error!(error = %error, "server exited with error");
            }
        }
        Err(error) => {
            error!(error = %error, "failed to bind server address");
            std::process::exit(1);
        }
    }
}
