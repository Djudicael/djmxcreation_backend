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
        let pg_port = env::var("PG_PORT")
            .expect("Cannot find PG_PORT")
            .parse::<u16>()
            .expect("PG_PORT must be a valid number");
        let database = DatabaseConfiguration::new(pg_host, pg_db, pg_user, pg_password, 5, pg_port);

        // Configuration for storage
        let minio_endpoint = env::var("MINIO_ENDPOINT").expect("Cannot find MINIO_ENDPOINT");
        let minio_access_key = env::var("MINIO_ACCESS_KEY").expect("Cannot find MINIO_ACCESS_KEY");
        let minio_secret_key = env::var("MINIO_SECRET_KEY").expect("Cannot find MINIO_SECRET_KEY");
        let region = env::var("MINIO_REGION").expect("Cannot find MINIO_REGION");
        let port = env::var("PORT").unwrap_or("8081".to_string());
        let minio_admin_endpoint =
            env::var("MINIO_ADMIN_ENDPOINT").expect("Cannot find MINIO_ADMIN_ENDPOINT");
        let minio_admin_token =
            env::var("MINIO_ADMIN_TOKEN").expect("Cannot find MINIO_ADMIN_TOKEN");

        let storage = StorageConfiguration::new(
            minio_endpoint,
            minio_access_key,
            minio_secret_key,
            region,
            minio_admin_endpoint,
            minio_admin_token,
        );

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
