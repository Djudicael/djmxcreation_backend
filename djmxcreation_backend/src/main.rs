use server::starter::start;
// use djmxcreation_backend;
mod app_error;
mod config;
mod controller;
mod domain;
mod mapper;
mod repository;
mod router;
mod server;
mod service;
mod view;

#[tokio::main]
async fn main() {
    // start the server

    match start().await {
        Ok(_) => println!("Server ended"),
        Err(ex) => println!("ERROR - web server failed to start. Cause {:?}", ex),
    }
}
