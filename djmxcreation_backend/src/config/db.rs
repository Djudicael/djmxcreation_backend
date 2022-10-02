use std::{env, time::Duration};

use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type Db = Pool<Postgres>;

struct DatabaseConfiguration {
    pg_host: String,
    pg_db: String,
    pg_user: String,
    pg_password: String,
    pg_app_max_con: u32,
}

impl DatabaseConfiguration {
    pub fn new(
        pg_host: &str,
        pg_db: &str,
        pg_user: &str,
        pg_password: &str,
        pg_app_max_con: u32,
    ) -> Self {
        Self {
            pg_host: pg_host.to_string(),
            pg_db: pg_db.to_string(),
            pg_user: pg_user.to_string(),
            pg_password: pg_password.to_string(),
            pg_app_max_con,
        }
    }
}

pub async fn init_db() -> Result<Db, sqlx::Error> {
    dotenv().unwrap();
    let pg_host = env::var("PG_HOST").unwrap();
    let pg_db = env::var("PG_DB").unwrap();
    let pg_user = env::var("PG_USER").unwrap();
    let pg_password = env::var("PG_PASSWORD").unwrap();

    let database = DatabaseConfiguration::new(
        pg_host.as_str(),
        pg_db.as_str(),
        pg_user.as_str(),
        pg_password.as_str(),
        5,
    );

    new_db_pool(&database).await
}

async fn new_db_pool(config: &DatabaseConfiguration) -> Result<Db, sqlx::Error> {
    let con_string = format!(
        "postgres://{}:{}@{}/{}",
        &config.pg_user.as_str(),
        &config.pg_password.as_str(),
        &config.pg_host.as_str(),
        &config.pg_db.as_str()
    );

    PgPoolOptions::new()
        .max_connections(config.pg_app_max_con)
        .connect_timeout(Duration::from_millis(500))
        .connect(&con_string)
        .await
}
