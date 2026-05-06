use std::sync::Arc;
use tokio::sync::OnceCell;

use app_config::database_configuration::DatabaseConfiguration;
use repository::config::db::DatabaseConfig;
use rustainers::images::Postgres;
use rustainers::runner::Runner;
use rustainers::ExposedPort;

use crate::rustfs::rustfs_endpoint;
use crate::test_db::run_migrations;

pub type PostgresContainer = rustainers::Container<Postgres>;

static POSTGRES: OnceCell<(PostgresContainer, Arc<DatabaseConfig>, String)> = OnceCell::const_new();

/// Shared RustFS endpoint — started once per test process.
static RUSTFS: OnceCell<String> = OnceCell::const_new();

pub async fn shared_postgres() -> (Arc<DatabaseConfig>, String) {
    let (_, config, uri) = POSTGRES
        .get_or_init(|| async {
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
                .with_port(ExposedPort::new(test_db_config.pg_port));

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

            let config = Arc::new(DatabaseConfig::new(&test_db_config).with_uri(&uri));

            run_migrations(&uri).await;

            (container, config, uri)
        })
        .await;

    (config.clone(), uri.clone())
}

pub async fn shared_rustfs() -> String {
    RUSTFS
        .get_or_init(|| async {
            let mut image = rustainers::images::GenericImage::new(
                rustainers::ImageName::new("docker.io/rustfs/rustfs"),
            );
            image.add_port_mapping(9000);
            let podman = Runner::podman().expect("Failed to create Podman runner");
            let container = podman
                .start(image)
                .await
                .expect("Failed to start RustFS container");

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let port = container
                .host_port(9000u16)
                .await
                .expect("Failed to get RustFS port");

            rustfs_endpoint(port.into())
        })
        .await
        .clone()
}
