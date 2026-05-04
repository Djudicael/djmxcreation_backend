use app_config::database_configuration::DatabaseConfiguration;
use repository::config::db::DatabaseConfig;
use rustainers::images::Postgres;
use rustainers::runner::Runner;
use rustainers::ExposedPort;
use std::sync::Arc;
use tokio::fs;

pub type PostgresContainer = rustainers::Container<Postgres>;

/// Start a PostgreSQL container via Podman and return it together with a
/// `DatabaseConfig` pointed at the container.
pub async fn start_postgres() -> (PostgresContainer, Arc<DatabaseConfig>, String) {
    let test_db_config = DatabaseConfiguration {
        pg_user: "postgres".to_string(),
        pg_password: "postgres".to_string(),
        pg_host: "localhost".to_string(),
        pg_db: "portfolio".to_string(),
        pg_app_max_con: 5,
        pg_port: 5432,
    };

    let image = Postgres::default()
        .with_db(test_db_config.pg_db.as_str())
        .with_user(test_db_config.pg_user.as_str())
        .with_password(test_db_config.pg_password.as_str())
        .with_port(ExposedPort::fixed(test_db_config.pg_port, test_db_config.pg_port));

    let podman = Runner::podman().expect("Failed to create Podman runner");
    let container = podman
        .start(image)
        .await
        .expect("Failed to start PostgreSQL container");

    // Wait for Postgres to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let uri = container
        .url()
        .await
        .expect("Failed to get container URL")
        .to_string();

    let config = Arc::new(DatabaseConfig::new(&test_db_config).with_uri(&uri));
    (container, config, uri)
}

/// Run all SQL migration files from `sql/migrations` against the given URI.
///
/// Files are executed in lexicographic order (V1__, V2__, …).
pub async fn run_migrations(uri: &str) {
    let mut entries = fs::read_dir("sql/migrations")
        .await
        .expect("Failed to read migrations directory");

    let mut files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            files.push(path);
        }
    }

    files.sort();

    let mut conn = DatabaseConfig::connect_str(uri)
        .await
        .expect("Failed to connect for migrations");

    for path in &files {
        let sql = fs::read_to_string(path)
            .await
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

        if sql.trim().is_empty() {
            continue;
        }

        // Split on DO $$ blocks and regular statements to handle them separately
        // For simplicity, just execute the whole SQL. Refinery-style migrations
        // usually have one statement per file, but some (like V1) have complex blocks.
        if let Err(e) = conn.execute(&sql).await {
            let msg = e.to_string();
            if msg.contains("duplicate_database") || msg.contains("already exists") {
                continue;
            }
            panic!("Migration failed for {}: {e}", path.display());
        }
    }
}

/// Convenience helper: start Postgres, run migrations, and return the config.
///
/// The returned `PostgresContainer` must be kept alive for the duration of the test.
pub async fn setup_test_db() -> (PostgresContainer, Arc<DatabaseConfig>) {
    let (container, config, uri) = start_postgres().await;
    run_migrations(&uri).await;
    (container, config)
}
