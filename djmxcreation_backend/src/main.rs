use djmxcreation_backend::*;
use server::starter::start;

#[tokio::main]
async fn main() {
    // start the server

    match start().await {
        Ok(_) => println!("Server ended"),
        Err(ex) => println!("ERROR - web server failed to start. Cause {:?}", ex),
    }
}
