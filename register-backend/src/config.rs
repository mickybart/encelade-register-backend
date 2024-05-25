//! Application Configuration

use std::env;

use config::{Config, ConfigError, File};
use serde::Deserialize;

/// The Application Configuration exposed
///
/// Configuration structure file expected:
///
/// ```yaml
/// listen: 'IP:PORT'
/// ```
#[derive(Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) listen: String,
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
