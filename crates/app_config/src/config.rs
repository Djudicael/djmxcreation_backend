use std::env;

use crate::{
    config_error::ConfigError, database_configuration::DatabaseConfiguration,
    storage_configuration::StorageConfiguration,
};
use dotenv::dotenv;

#[derive(Default)]
pub struct Config {
    pub database: DatabaseConfiguration,
    pub storage: StorageConfiguration,
    pub port: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Returns `Err(ConfigError)` if any required variable is missing or invalid,
    /// so the caller can handle startup failure cleanly instead of panicking.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        // ── Database ────────────────────────────────────────────────────────
        let pg_host = required("PG_HOST")?;
        let pg_db = required("PG_DB")?;
        let pg_user = required("PG_USER")?;
        let pg_password = required("PG_PASSWORD")?;
        let pg_port = required("PG_PORT")?
            .parse::<u16>()
            .map_err(|e| ConfigError::Invalid {
                key: "PG_PORT",
                reason: e.to_string(),
            })?;
        let pg_max_con = env::var("PG_APP_MAX_CON")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5);

        let database =
            DatabaseConfiguration::new(pg_host, pg_db, pg_user, pg_password, pg_max_con, pg_port);

        // ── Object storage (RustFS / S3-compatible) ─────────────────────────
        let storage_endpoint = required("STORAGE_ENDPOINT")?;
        let storage_access_key = required("STORAGE_ACCESS_KEY")?;
        let storage_secret_key = required("STORAGE_SECRET_KEY")?;
        let storage_region = env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let storage_bucket = env::var("STORAGE_BUCKET").unwrap_or_else(|_| "portfolio".to_string());

        let storage = StorageConfiguration::new(
            storage_endpoint,
            storage_access_key,
            storage_secret_key,
            storage_region,
            storage_bucket,
        );

        // ── Server ──────────────────────────────────────────────────────────
        let port = env::var("PORT").unwrap_or_else(|_| "8081".to_string());

        Ok(Self {
            database,
            storage,
            port,
        })
    }

    pub fn get_storage(&self) -> StorageConfiguration {
        self.storage.clone()
    }
}

fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::Missing(key))
}
