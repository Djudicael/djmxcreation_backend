use app_config::database_configuration::DatabaseConfiguration;

use refinery::embed_migrations;
use tokio_postgres::{Error, NoTls};

pub type ClientV2 = tokio_postgres::Client;
embed_migrations!("../../sql/migrations");

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
    // println!(
    //     "Found migrations: {:?}",
    //     migrations::runner().get_migrations()
    // );
    let _ = migrations::runner()
        .run_async(client)
        .await
        .expect("Data base Migration error");

    println!("Migrations applied successfully.");
    Ok(())
}
