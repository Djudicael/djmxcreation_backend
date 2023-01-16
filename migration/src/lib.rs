use std::{fs, path::PathBuf, time::Duration};

use app_config::database_configuration::DatabaseConfiguration;
use base64ct::{Base64, Encoding};
use sha3::{Digest, Sha3_256};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::log::info;

pub type Db = Pool<Postgres>;

// sql files
const SQL_DIR: &str = "./sql/migration";

fn content_hasher(content: &str) -> String {
    // create a SHA3-256 object
    let mut hasher = Sha3_256::default();

    // write input message
    hasher.update(content.as_bytes());

    // read hash digest
    let hash = hasher.finalize();
    Base64::encode_string(&hash)
}

pub async fn init_db_migration(database: &DatabaseConfiguration) -> Result<(), sqlx::Error> {
    println!("Initializing DB migration");
    // run the app sql files
    let app_db = pool(database).await?;

    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        // .into_iter()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();
    // execute  each file
    for path in paths {
        if let Some(path) = path.to_str() {
            // only .sql and not the recreate
            if path.ends_with(".sql") {
                pexec(&app_db, path).await?;
            }
        }
    }

    Ok(())
}

pub async fn init_db(config: &DatabaseConfiguration) -> Result<Db, sqlx::Error> {
    pool(config).await
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    // read the file

    println!("Executing SQL file: {}", file);

    let content = fs::read_to_string(file).map_err(|ex| {
        info!("ERROR reading {} ( cause: {:?})", file, ex);
        ex
    })?;

    let _hashed_content = content_hasher(content.as_str());
    // dbg!(&content);

    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        match sqlx::query(sql).execute(db).await {
            Ok(_) => (),
            Err(ex) => info!(
                "WARNING -pexex - SQL file '{}' FAILED caused: {} ",
                file, ex
            ),
        }
    }
    Ok(())
}

async fn pool(config: &DatabaseConfiguration) -> Result<Db, sqlx::Error> {
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
