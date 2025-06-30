use axum::body::Body;
use axum::http::{Response, StatusCode, Uri};
use axum::{Router, response::IntoResponse};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Chemin vers le dossier statique
    let target = env::args().nth(1).unwrap_or_else(|| "admin".to_string());
    let public_dir = format!("front/{target}");

    // Créer un handler pour toutes les routes qui tente d'abord de servir un fichier, puis revient à index.html
    async fn spa_handler(uri: Uri, dir: String) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };

        let full_path = format!("{}/{}", dir, path);

        // Vérifier si le fichier existe
        if let Ok(content) = tokio::fs::read(&full_path).await {
            // Déterminer le Content-Type en fonction de l'extension
            let content_type = match path.split('.').last() {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(content))
                .unwrap()
        } else {
            // Servir index.html comme fallback
            if let Ok(content) = tokio::fs::read(format!("{}/index.html", dir)).await {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(content))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("index.html non trouvé"))
                    .unwrap()
            }
        }
    }

    // Create a fallback handler that's cloneable
    let handler = move |uri: Uri| {
        let dir = public_dir.clone();
        async move { spa_handler(uri, dir).await }
    };

    // Router principal qui traite toutes les routes
    let app = Router::new().fallback(handler);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 SPA server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
