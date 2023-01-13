use std::time::Duration;

use app_config::database_configuration::DatabaseConfiguration;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

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
