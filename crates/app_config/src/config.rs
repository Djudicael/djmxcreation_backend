use std::env;

use crate::{
    database_configuration::DatabaseConfiguration, security_config::SecurityConfig,
    storage_configuration::StorageConfiguration,
};
use dotenv::dotenv;

#[derive(Default)]
pub struct Config {
    pub database: DatabaseConfiguration,
    pub storage: StorageConfiguration,
    pub port: String,
    pub security: SecurityConfig,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        // ── Database ────────────────────────────────────────────────────────
        let pg_host = env::var("PG_HOST").expect("PG_HOST must be set");
        let pg_db = env::var("PG_DB").expect("PG_DB must be set");
        let pg_user = env::var("PG_USER").expect("PG_USER must be set");
        let pg_password = env::var("PG_PASSWORD").expect("PG_PASSWORD must be set");
        let pg_port = env::var("PG_PORT")
            .expect("PG_PORT must be set")
            .parse::<u16>()
            .expect("PG_PORT must be a valid port number (0–65535)");
        let pg_max_con = env::var("PG_APP_MAX_CON")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5);

        let database =
            DatabaseConfiguration::new(pg_host, pg_db, pg_user, pg_password, pg_max_con, pg_port);

        // ── Object storage (RustFS / S3-compatible) ─────────────────────────
        let storage_endpoint =
            env::var("STORAGE_ENDPOINT").expect("STORAGE_ENDPOINT must be set");
        let storage_access_key =
            env::var("STORAGE_ACCESS_KEY").expect("STORAGE_ACCESS_KEY must be set");
        let storage_secret_key =
            env::var("STORAGE_SECRET_KEY").expect("STORAGE_SECRET_KEY must be set");
        let storage_region =
            env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let storage_bucket =
            env::var("STORAGE_BUCKET").unwrap_or_else(|_| "portfolio".to_string());

        let storage = StorageConfiguration::new(
            storage_endpoint,
            storage_access_key,
            storage_secret_key,
            storage_region,
            storage_bucket,
        );

        // ── API security (basic auth) ────────────────────────────────────────
        let username = env::var("USERNAME_APP").expect("USERNAME_APP must be set");
        let password = env::var("PASSWORD_APP").expect("PASSWORD_APP must be set");
        let security = SecurityConfig::new(username, password);

        // ── Server ──────────────────────────────────────────────────────────
        let port = env::var("PORT").unwrap_or_else(|_| "8081".to_string());

        Self {
            database,
            storage,
            port,
            security,
        }
    }

    pub fn get_storage(&self) -> StorageConfiguration {
        self.storage.clone()
    }

    pub fn get_security(&self) -> SecurityConfig {
        self.security.clone()
    }
}
