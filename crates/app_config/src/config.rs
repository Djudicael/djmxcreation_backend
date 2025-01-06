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
        // Configuration for database
        let pg_host = env::var("PG_HOST").expect("Cannot find PG_HOST");
        let pg_db = env::var("PG_DB").expect("Cannot find PG_DB");
        let pg_user = env::var("PG_USER").expect("Cannot find");
        let pg_password = env::var("PG_PASSWORD").expect("Cannot find");
        let database = DatabaseConfiguration::new(pg_host, pg_db, pg_user, pg_password, 5);

        // Configuration for storage
        let minio_endpoint = env::var("MINIO_ENDPOINT").expect("Cannot find MINIO_ENDPOINT");
        let minio_access_key = env::var("MINIO_ACCESS_KEY").expect("Cannot find MINIO_ACCESS_KEY");
        let minio_secret_key = env::var("MINIO_SECRET_KEY").expect("Cannot find MINIO_SECRET_KEY");
        let region = env::var("MINIO_REGION").expect("Cannot find MINIO_REGION");
        let port = env::var("PORT").unwrap_or("8081".to_string());

        let storage =
            StorageConfiguration::new(minio_endpoint, minio_access_key, minio_secret_key, region);

        let username = env::var("USERNAME_APP").expect("Cannot find USERNAME");
        let password = env::var("PASSWORD_APP").expect("Cannot find PASSWORD");

        let security = SecurityConfig::new(username, password);

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
