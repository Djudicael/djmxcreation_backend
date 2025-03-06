use std::error::Error as StdError;
use std::fmt;

use app_config::database_configuration::DatabaseConfiguration;

use deadpool_postgres::{Config, CreatePoolError, Pool, PoolError, Runtime};

use refinery::embed_migrations;
use tokio_postgres::{Error, NoTls};

pub type ClientV2 = tokio_postgres::Client;
embed_migrations!("../../sql/migrations");

#[derive(Debug)]
pub enum DatabaseError {
    Pool(CreatePoolError),
    PoolConnection(PoolError),
    Connection(Error),
    Migration(Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Pool(e) => write!(f, "Pool creation error: {}", e),
            DatabaseError::PoolConnection(e) => write!(f, "Pool connection error: {}", e),
            DatabaseError::Connection(e) => write!(f, "Database connection error: {}", e),
            DatabaseError::Migration(e) => write!(f, "Database migration error: {}", e),
        }
    }
}

impl StdError for DatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            DatabaseError::Pool(e) => Some(e),
            DatabaseError::PoolConnection(e) => Some(e),
            DatabaseError::Connection(e) => Some(e),
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
        let mut cfg = Config::new();

        if let Some(url) = database_url {
            cfg.url = Some(url.to_string());
        } else {
            cfg.host = Some(config.pg_host.clone());
            cfg.user = Some(config.pg_user.clone());
            cfg.password = Some(config.pg_password.clone());
            cfg.dbname = Some(config.pg_db.clone());
            cfg.port = Some(config.pg_port);
        }

        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(DatabaseError::Pool)?;

        // Apply migrations using raw connection
        let mut conn = pool.get().await.map_err(DatabaseError::PoolConnection)?;

        // Get mutable reference to the underlying client
        let client = conn.as_mut();
        apply_migrations(client)
            .await
            .map_err(DatabaseError::Migration)?;

        Ok(Self { pool })
    }

    pub async fn get_client(&self) -> Result<deadpool_postgres::Client, PoolError> {
        self.pool.get().await
    }
}

pub async fn db_client(
    config: &DatabaseConfiguration,
    database_url: Option<&str>,
) -> Result<ClientV2, Error> {
    let con_string = format!(
        "postgres://{}:{}@{}/{}",
        &config.pg_user.as_str(),
        &config.pg_password.as_str(),
        &config.pg_host.as_str(),
        &config.pg_db.as_str()
    );

    let (mut client, connection) =
        tokio_postgres::connect(database_url.unwrap_or_else(|| &con_string), NoTls).await?;
    // Spawn connection in background to handle messages
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Apply migrations
    apply_migrations(&mut client).await?;

    Ok(client)
}

async fn apply_migrations(client: &mut ClientV2) -> Result<(), Error> {
    let _ = migrations::runner()
        .run_async(client)
        .await
        .expect("Database Migration error");

    println!("Migrations applied successfully.");
    Ok(())
}
