#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
mod config;
mod errors;
mod io_schema;
mod mqtt_client;
mod repository;
mod service;
mod topics;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info,poem=info");
    tracing_subscriber::fmt::init();
    service::run().await;
}
