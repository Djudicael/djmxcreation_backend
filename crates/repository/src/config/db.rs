use std::time::Duration;

use app_config::database_configuration::DatabaseConfiguration;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio_postgres::{tls::NoTlsStream, Error, NoTls};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool(config: &DatabaseConfiguration) -> Result<Db, sqlx::Error> {
    let con_string = format!(
        "postgres://{}:{}@{}/{}",
        &config.pg_user.as_str(),
        &config.pg_password.as_str(),
        &config.pg_host.as_str(),
        &config.pg_db.as_str()
    );

    PgPoolOptions::new()
        .max_connections(config.pg_app_max_con)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&con_string)
        .await
}

pub type ClientV2 = tokio_postgres::Client;

pub type DbV2 = (
    ClientV2,
    tokio_postgres::Connection<tokio_postgres::Socket, NoTlsStream>,
);

pub async fn db_client(config: &DatabaseConfiguration) -> Result<DbV2, Error> {
    let con_string = format!(
        "postgres://{}:{}@{}/{}",
        &config.pg_user.as_str(),
        &config.pg_password.as_str(),
        &config.pg_host.as_str(),
        &config.pg_db.as_str()
    );
    let (client, connection) = tokio_postgres::connect(&con_string, NoTls).await?;

    Ok((client, connection))
}
