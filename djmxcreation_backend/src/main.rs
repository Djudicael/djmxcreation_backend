use server::starter::start;

// use djmxcreation_backend;
mod router;
mod server;
mod controller;
mod config;
mod domain;
mod repository;
mod service;
mod app_error;


#[tokio::main]
async fn main() {
   // start the server
	match start().await {
		Ok(_) => println!("Server ended"),
		Err(ex) => println!("ERROR - web server failed to start. Cause {:?}", ex),
	}
}
