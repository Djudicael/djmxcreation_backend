use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::{Router, extract::OriginalUri, response::IntoResponse};
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let target = env::args().nth(1).unwrap_or_else(|| "admin".to_string());
    let public_dir = format!("front/{target}");

    async fn spa_handler(path: String, dir: String) -> impl IntoResponse {
        println!("[spa_handler] path: {} dir: {}", path, dir);
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
            println!("[spa_handler] Detected asset, returning 404");
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap();
        }
        // Always serve index.html for fallback
        let index_path = format!("{}/index.html", dir);
        println!("[spa_handler] Serving index.html from: {}", index_path);
        match tokio::fs::read(&index_path).await {
            Ok(content) => {
                println!("[spa_handler] index.html found and served");
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(content))
                    .unwrap()
            }
            Err(e) => {
                println!("[spa_handler] ERROR reading index.html: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("index.html non trouvé"))
                    .unwrap()
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
    println!("🚀 SPA server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
