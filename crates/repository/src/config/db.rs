use std::error::Error as StdError;
use std::fmt;

use app_config::database_configuration::DatabaseConfiguration;
use deadpool_postgres::{Config, CreatePoolError, Pool, PoolError, Runtime};
use refinery::embed_migrations;
use tokio_postgres::NoTls;
use tracing::info;

pub type ClientV2 = tokio_postgres::Client;
embed_migrations!("../../sql/migrations");

#[derive(Debug)]
pub enum DatabaseError {
    Pool(CreatePoolError),
    PoolConnection(PoolError),
    Migration(refinery::Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Pool(e) => write!(f, "pool creation error: {e}"),
            DatabaseError::PoolConnection(e) => write!(f, "pool connection error: {e}"),
            DatabaseError::Migration(e) => write!(f, "migration error: {e}"),
        }
    }
}

impl StdError for DatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            DatabaseError::Pool(e) => Some(e),
            DatabaseError::PoolConnection(e) => Some(e),
            DatabaseError::Migration(e) => Some(e),
        }
    }
}

pub struct DatabasePool {
    pool: Pool,
}

impl DatabasePool {
    pub async fn new(
        config: &DatabaseConfiguration,
        database_url: Option<&str>,
    ) -> Result<Self, DatabaseError> {
        let pool = if let Some(url) = database_url {
            let tokio_config: tokio_postgres::Config = url.parse().expect("Invalid database URL");
            let mgr = deadpool_postgres::Manager::new(tokio_config, NoTls);
            Pool::builder(mgr)
                .runtime(Runtime::Tokio1)
                .build()
                .expect("Failed to build database pool from URL")
        } else {
            let mut cfg = Config::new();
            cfg.host = Some(config.pg_host.clone());
            cfg.user = Some(config.pg_user.clone());
            cfg.password = Some(config.pg_password.clone());
            cfg.dbname = Some(config.pg_db.clone());
            cfg.port = Some(config.pg_port);

            cfg.create_pool(Some(Runtime::Tokio1), NoTls)
                .map_err(DatabaseError::Pool)?
        };

        let mut conn = pool.get().await.map_err(DatabaseError::PoolConnection)?;
        apply_migrations(conn.as_mut())
            .await
            .map_err(DatabaseError::Migration)?;

        Ok(Self { pool })
    }

    pub async fn get_client(&self) -> Result<deadpool_postgres::Client, PoolError> {
        self.pool.get().await
    }
}

async fn apply_migrations(client: &mut ClientV2) -> Result<(), refinery::Error> {
    migrations::runner().run_async(client).await?;
    info!("database migrations applied successfully");
    Ok(())
}
