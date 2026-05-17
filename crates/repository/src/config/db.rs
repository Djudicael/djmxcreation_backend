use app_config::database_configuration::DatabaseConfiguration;
use wasi_pg_client::{Config, Connection};

/// Holds PostgreSQL connection URL.
#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn new(config: &DatabaseConfiguration) -> Self {
        Self {
            url: config.url.clone(),
        }
    }

    /// Open a fresh connection to PostgreSQL.
    pub async fn connect(&self) -> Result<Connection, wasi_pg_client::PgError> {
        let cfg = Config::from_uri(&self.url)
            .map_err(|e| wasi_pg_client::PgError::Config(e.to_string()))?;
        Connection::connect(&cfg).await
    }

    /// Open a connection from a PostgreSQL URI (used by integration tests).
    pub async fn connect_str(uri: &str) -> Result<Connection, wasi_pg_client::PgError> {
        let cfg =
            Config::from_uri(uri).map_err(|e| wasi_pg_client::PgError::Config(e.to_string()))?;
        Connection::connect(&cfg).await
    }
}
