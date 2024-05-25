use std::env;

use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) listen: String,
}

impl AppConfig {
    pub(crate) fn build() -> Result<Self, ConfigError> {
        let profile = env::var("REGISTER_PROFILE").unwrap_or("prod".to_owned());

        Config::builder()
            .add_source(File::with_name(&format!("config/{}.yaml", profile)).required(false))
            .add_source(File::with_name("config/local.yaml").required(false))
            .build()?
            .try_deserialize()
    }
}
