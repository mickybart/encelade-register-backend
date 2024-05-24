#[macro_use]
extern crate num_derive;

use std::error::Error;

use register::{Register, RegisterServer};
use tonic::transport::Server;

mod register;
mod mongodb;
mod observability;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    observability::init_tracing();

    let addr = "0.0.0.0:50051".parse()?;
    let register_service = Register::default();

    tracing::info!("listening on {}", addr);

    Server::builder()
        .add_service(RegisterServer::new(register_service))
        .serve(addr)
        .await?;

    Ok(())
}
