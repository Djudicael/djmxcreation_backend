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
    /// Allowed CORS origins (comma-separated). Empty means no origins allowed.
    pub cors_origins: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Primary: `DATABASE_URL` env var.
    /// Fallback: `PG_HOST`, `PG_PORT`, `PG_DB`, `PG_USER`, `PG_PASSWORD` (for backward compat).
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        // ── Database ────────────────────────────────────────────────────────
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let host = env::var("PG_HOST").unwrap_or_else(|_| "localhost".into());
            let port = env::var("PG_PORT").unwrap_or_else(|_| "5432".into());
            let db = env::var("PG_DB").unwrap_or_else(|_| "portfolio".into());
            let user = env::var("PG_USER").unwrap_or_else(|_| "postgres".into());
            let password = env::var("PG_PASSWORD").unwrap_or_else(|_| "postgres".into());
            format!("postgresql://{user}:{password}@{host}:{port}/{db}")
        });

        let pg_max_con = env::var("PG_APP_MAX_CON")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5);

        let database = DatabaseConfiguration::new(database_url, pg_max_con);

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

        // ── CORS ────────────────────────────────────────────────────────────
        let cors_origins: Vec<String> = env::var("CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database,
            storage,
            port,
            cors_origins,
        })
    }

    pub fn get_storage(&self) -> StorageConfiguration {
        self.storage.clone()
    }
}

fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::Missing(key))
}
