#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

//! Register Service exposed by grpc and grpc-web

#[macro_use]
extern crate num_derive;

use std::error::Error;

use register::{Register, RegisterServer};
use tonic::transport::Server;

use crate::config::AppConfig;

mod config;
mod mongodb;
mod observability;
mod register;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    observability::init_tracing();

    let app_config: AppConfig = AppConfig::build()?;

    let addr = app_config.listen.parse()?;
    let register_server = RegisterServer::new(Register::default());

    tracing::info!("listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(register_server))
        .serve(addr)
        .await?;

    Ok(())
}
