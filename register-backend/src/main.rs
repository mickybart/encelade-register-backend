#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

//! Register Service exposed by grpc and grpc-web

#[macro_use]
extern crate num_derive;

use std::error::Error;

use register::{Register, RegisterServer};
use tonic::transport::{Identity, Server, ServerTlsConfig};

use crate::config::AppConfig;

mod config;
mod mongodb;
mod observability;
mod register;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    observability::init_tracing();

    let app_config: AppConfig = AppConfig::build()?;

    let addr = app_config.service.listen.parse()?;
    let register_server = RegisterServer::new(Register::new(app_config.mongodb).await?);

    tracing::info!("listening on {}", addr);

    if app_config.service.tls {
        let cert = tokio::fs::read("config/server.crt").await?;
        let key = tokio::fs::read("config/server.key").await?;
        let identity = Identity::from_pem(cert, key);

        tracing::info!("tls enabled");

        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(tonic_web::enable(register_server))
            .serve(addr)
            .await?;
    } else {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(register_server))
            .serve(addr)
            .await?;
    }

    Ok(())
}
