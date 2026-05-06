use app_config::database_configuration::DatabaseConfiguration;
use wasi_pg_client::{Config, Connection};

/// Holds PostgreSQL connection parameters.
///
/// Since pgbouncer manages connection pooling server-side we simply open a
/// fresh `Connection` for every database operation.
#[derive(Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    /// Optional full URI override (e.g. for tests connecting to containers).
    pub uri: Option<String>,
}

impl DatabaseConfig {
    pub fn new(config: &DatabaseConfiguration) -> Self {
        Self {
            host: config.pg_host.clone(),
            port: config.pg_port,
            user: config.pg_user.clone(),
            password: config.pg_password.clone(),
            dbname: config.pg_db.clone(),
            uri: None,
        }
    }

    /// Attach a connection URI so `connect()` uses it instead of host:port fields.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Open a fresh connection to PostgreSQL (via pgbouncer).
    pub async fn connect(&self) -> Result<Connection, wasi_pg_client::PgError> {
        let cfg = if let Some(ref uri) = self.uri {
            Config::from_uri(uri).map_err(|e| wasi_pg_client::PgError::Config(e.to_string()))?
        } else {
            Config::new()
                .host(&self.host)
                .port(self.port)
                .user(&self.user)
                .password(&self.password)
                .database(&self.dbname)
        };

        Connection::connect(&cfg).await
    }

    /// Open a connection from a PostgreSQL URI (used by integration tests).
    pub async fn connect_str(uri: &str) -> Result<Connection, wasi_pg_client::PgError> {
        let cfg =
            Config::from_uri(uri).map_err(|e| wasi_pg_client::PgError::Config(e.to_string()))?;
        Connection::connect(&cfg).await
    }
}
