#[macro_use]
extern crate num_derive;

use std::error::Error;

use register::{Register, RegisterServer};
use tonic::transport::Server;

use crate::config::AppConfig;

mod register;
mod mongodb;
mod observability;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    observability::init_tracing();

    let app_config = AppConfig::build()?;

    let addr = app_config.listen.grpc.parse()?;
    let register_service = Register::default();

    tracing::info!("listening on {}", addr);

    Server::builder()
        .add_service(RegisterServer::new(register_service))
        .serve(addr)
        .await?;

    Ok(())
}
