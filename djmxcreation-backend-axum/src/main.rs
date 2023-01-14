use djmxcreation_backend_axum::*;
use server::starter::start;

#[tokio::main]
async fn main() {
    match start().await {
        Ok(_) => println!("Server ended"),
        Err(ex) => println!("ERROR - web server failed to start. Cause {ex:?}"),
    }
}
