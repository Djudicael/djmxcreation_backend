use djmxcreation_backend_axum::*;
use server::starter::start;

#[tokio::main]
async fn main() {
    // Add backtrace for debug purposes
    std::env::set_var("RUST_BACKTRACE", "1");
    println!(
        "printing backtrace: {}",
        std::backtrace::Backtrace::capture()
    );

    match start().await {
        Ok(_) => println!("Server ended"),
        Err(ex) => println!("ERROR - web server failed to start. Cause {ex:?}"),
    }
}
