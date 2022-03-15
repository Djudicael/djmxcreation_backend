use std::{env, fs, path::PathBuf, time::Duration};

use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type Db = Pool<Postgres>;

// sql files
const SQL_DIR: &str = "../sql/";

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
        &pg_host.as_str(),
        &pg_db.as_str(),
        &pg_user.as_str(),
        &pg_password.as_str(),
        5,
    );

    // run the app sql files
    // let app_db = new_db_pool(&database).await?;
    // let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
    //     .into_iter()
    //     .filter_map(|e| e.ok().map(|e| e.path()))
    //     .collect();

    // paths.sort();

    // // execute  each file
    // for path in paths {
    //     if let Some(path) = path.to_str() {
    //         // only .sql and not the recreate
    //         if path.ends_with(".sql") {
    //             pexec(&app_db, &path).await?;
    //         }
    //     }
    // }

    // returning the app db
    new_db_pool(&database).await
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    // read the file

    let content = fs::read_to_string(file).map_err(|ex| {
        println!("ERROR reading {} ( cause: {:?})", file, ex);
        ex
    })?;

    let sqls: Vec<&str> = content.split(";").collect();

    for sql in sqls {
        match sqlx::query(&sql).execute(db).await {
            Ok(_) => (),
            Err(ex) => println!(
                "WARNING -pexex - SQL file '{}' FAILED caused: {} ",
                file, ex
            ),
        }
    }
    Ok(())
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
