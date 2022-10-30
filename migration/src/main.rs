use migration::{self, init_db_migration_test, DatabaseConfiguration};

#[tokio::main]
async fn main() {
    let database = DatabaseConfiguration::new("127.0.0.1", "portfolio", "postgres", "postgres", 5);

    match init_db_migration_test(&database).await {
        Ok(_) => println!("Server ended"),
        Err(ex) => println!("ERROR - web server failed to start. Cause {:?}", ex),
    }
}
