//! Service
//! 
//! Build and run the service
//! 
//! Support:
//! - Cors
//! - Auth (POC)
//! - TLS
//! - Grpc-web

use std::{error::Error, sync::Arc, time::Duration};

use http::{HeaderName, Method};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    auth::Auth,
    config::AppConfig,
    observability,
    register::{Register, RegisterServer},
};

pub(crate) async fn run() -> Result<(), Box<dyn Error>> {
    observability::init_tracing();

    let config = AppConfig::build()?;

    let auth = Arc::new(Auth::new(config.service.tokens.unwrap_or(vec![]))?);

    let service =
        RegisterServer::with_interceptor(Register::new(config.mongodb).await?, move |req| {
            auth.check_auth(req)
        });

    let addr = config.service.listen.parse()?;

    tracing::info!("use tls: {}", config.service.tls);
    tracing::info!("listening on {}", addr);

    if config.service.tls {
        let cert = tokio::fs::read("config/server.crt").await?;
        let key = tokio::fs::read("config/server.key").await?;
        let identity = Identity::from_pem(cert, key);

        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .layer(cors_layer())
            .layer(GrpcWebLayer::new())
            .add_service(service)
            .serve(addr)
            .await?;
    } else {
        Server::builder()
            .accept_http1(true)
            .layer(cors_layer())
            .layer(GrpcWebLayer::new())
            .add_service(service)
            .serve(addr)
            .await?;
    }

    Ok(())
}

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin"];
const DEFAULT_ALLOW_HEADERS: [&str; 5] = [
    "x-grpc-web",
    "content-type",
    "x-user-agent",
    "grpc-timeout",
    "apikey",
];

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        // Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Origin: *`
        //.allow_credentials(true)
        .allow_methods([Method::GET, Method::POST])
        .max_age(DEFAULT_MAX_AGE)
        .expose_headers(
            DEFAULT_EXPOSED_HEADERS
                .iter()
                .cloned()
                .map(HeaderName::from_static)
                .collect::<Vec<HeaderName>>(),
        )
        .allow_headers(
            DEFAULT_ALLOW_HEADERS
                .iter()
                .cloned()
                .map(HeaderName::from_static)
                .collect::<Vec<HeaderName>>(),
        )
}
