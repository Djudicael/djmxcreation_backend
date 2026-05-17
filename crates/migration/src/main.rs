//! Database migration runner — native only.
//!
//! Reads SQL files from `sql/migrations/` and applies them in order.
//! Usage: DATABASE_URL=postgresql://... cargo run -p migration

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("PG_HOST").map(|_| {
            let host = std::env::var("PG_HOST").unwrap();
            let port = std::env::var("PG_PORT").unwrap_or_else(|_| "5432".into());
            let db = std::env::var("PG_DB").unwrap_or_else(|_| "portfolio".into());
            let user = std::env::var("PG_USER").unwrap_or_else(|_| "postgres".into());
            let password = std::env::var("PG_PASSWORD").unwrap_or_else(|_| "".into());
            format!("postgresql://{user}:{password}@{host}:{port}/{db}?sslmode=prefer")
        }).unwrap_or_else(|_| "postgresql://localhost:5432/portfolio?sslmode=prefer".into())
    });

    migration::run_migrations(&database_url).await
}
