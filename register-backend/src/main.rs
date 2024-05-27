#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

//! Register Service exposed by grpc and grpc-web

use std::process::ExitCode;

#[macro_use]
extern crate num_derive;

mod auth;
mod config;
mod mongodb;
mod observability;
mod register;
mod service;

#[tokio::main]
async fn main() -> ExitCode {
    let terminated = service::run().await;

    match terminated {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e.as_ref());

            ExitCode::FAILURE
        },
    }
}
