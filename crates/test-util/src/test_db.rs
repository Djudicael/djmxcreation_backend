use repository::config::db::DatabaseConfig;
use rustainers::images::Postgres;
use rustainers::runner::Runner;
use rustainers::ExposedPort;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

pub type PostgresContainer = rustainers::Container<Postgres>;

const TEST_DB_USER: &str = "postgres";
const TEST_DB_PASSWORD: &str = "postgres";
const TEST_DB_NAME: &str = "portfolio";
const TEST_DB_PORT: u16 = 5432;

fn find_migrations_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("..").join("..").join("sql").join("migrations"),
        manifest_dir.join("..").join("sql").join("migrations"),
        manifest_dir.join("sql").join("migrations"),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    let mut current = std::env::current_dir().unwrap_or_else(|_| manifest_dir.clone());
    loop {
        let migrations = current.join("sql").join("migrations");
        if migrations.exists() {
            return migrations;
        }
        if !current.pop() {
            break;
        }
    }
    panic!("Could not find sql/migrations directory. Searched from {:?}", std::env::current_dir());
}

/// Start a PostgreSQL container via Podman and return it together with a
/// `DatabaseConfig` pointed at the container.
pub async fn start_postgres() -> (PostgresContainer, Arc<DatabaseConfig>, String) {
    let image = Postgres::default()
        .with_db(TEST_DB_NAME)
        .with_user(TEST_DB_USER)
        .with_password(TEST_DB_PASSWORD)
        .with_port(ExposedPort::fixed(TEST_DB_PORT, TEST_DB_PORT));

    let podman = Runner::podman().expect("Failed to create Podman runner");
    let container = podman
        .start(image)
        .await
        .expect("Failed to start PostgreSQL container");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let uri = container
        .url()
        .await
        .expect("Failed to get container URL")
        .to_string();

    let uri = if uri.contains('?') {
        format!("{uri}&sslmode=disable")
    } else {
        format!("{uri}?sslmode=disable")
    };

    let config = Arc::new(DatabaseConfig { url: uri.clone() });
    (container, config, uri)
}

/// Run all SQL migration files from `sql/migrations` against the given URI.
pub async fn run_migrations(uri: &str) {
    let migrations_dir = find_migrations_dir();

    let mut entries = fs::read_dir(&migrations_dir)
        .await
        .unwrap_or_else(|e| panic!("Failed to read migrations directory {}: {e}", migrations_dir.display()));

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

        if let Err(e) = conn.execute(&sql).await {
            let msg = e.to_string();
            if msg.contains("duplicate") || msg.contains("already exists") {
                continue;
            }
            panic!("Migration failed for {}: {e}", path.display());
        }
    }
}

/// Convenience helper: start Postgres, run migrations, and return the config.
pub async fn setup_test_db() -> (PostgresContainer, Arc<DatabaseConfig>) {
    let (container, config, uri) = start_postgres().await;
    run_migrations(&uri).await;
    (container, config)
}
