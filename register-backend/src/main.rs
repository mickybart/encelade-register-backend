#[macro_use]
extern crate num_derive;

use std::error::Error;

use register::{Register, RegisterServer};
use tonic::transport::Server;

mod register;
mod mongodb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let register_service = Register::default();

    Server::builder()
        .add_service(RegisterServer::new(register_service))
        .serve(addr)
        .await?;

    Ok(())
}
