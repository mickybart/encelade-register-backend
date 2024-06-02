//! Application Configuration

use std::env;

use config::{Config, ConfigError, File};
use serde::Deserialize;

/// The Application Configuration exposed
///
/// Configuration structure file expected:
///
/// ```yaml
/// service:
///     # ip:port binding
///     listen: '127.0.0.1:50051'
/// 
///     # enable or disable tls
///     # if enabled, needs server.crt and server.key files
///     tls: false
/// 
///     # list of token. for demonstration purpose only !
///     # auth is dicabled if list is null or empty
///     tokens: []
/// mongodb:
///     # Mongodb uri with options
///     uri: 'mongodb://user:password@127.0.0.1:27017/'
/// 
///     # database name
///     db: 'encelade'
/// 
///     # collection name
///     collection: 'register'
/// ```
#[derive(Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) service: ServiceConfig,
    pub(crate) mongodb: MongoDbConfig,
}

#[derive(Deserialize)]
pub(crate) struct ServiceConfig {
    pub(crate) listen: String,
    pub(crate) tls: bool,
    pub(crate) tokens: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct MongoDbConfig {
    pub(crate) uri: String,
    pub(crate) db: String,
    pub(crate) collection: String,
}

impl AppConfig {
    /// build an application configuration
    ///
    /// load prod profile (default) and local if present.
    ///
    /// Use REGISTER_PROFILE environment variable if you want to use an alternative to prod profile.
    ///
    /// ```bash
    /// # load dev.yaml and local.yaml if present
    /// export REGISTER_PROFILE=dev
    /// ```
    pub(crate) fn build() -> Result<Self, ConfigError> {
        let profile = env::var("REGISTER_PROFILE").unwrap_or("prod".to_owned());

        Config::builder()
            .add_source(File::with_name(&format!("config/{}.yaml", profile)).required(false))
            .add_source(File::with_name("config/local.yaml").required(false))
            .build()?
            .try_deserialize()
    }
}
