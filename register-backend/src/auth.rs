//! Authentication
//!
//! Used to intercept request and validate token.

use tonic::{
    metadata::{errors::InvalidMetadataValue, Ascii, MetadataValue},
    Request, Status,
};

static AUTH_KEY: &str = "apikey";

pub(crate) struct Auth {
    tokens: Vec<MetadataValue<Ascii>>,
}

impl Auth {
    pub(crate) fn new(config: Vec<String>) -> Result<Self, InvalidMetadataValue> {
        let mut tokens: Vec<MetadataValue<Ascii>> = Vec::with_capacity(config.len());

        for token in config {
            tokens.push(token.parse()?);
        }

        tracing::info!("use auth: {}", !tokens.is_empty());

        Ok(Self { tokens })
    }

    pub(crate) fn check_auth(&self, req: Request<()>) -> Result<Request<()>, Status> {
        let token = req.metadata().get(AUTH_KEY);

        match token {
            None if self.tokens.is_empty() => Ok(req),
            Some(_) if self.tokens.is_empty() => Ok(req),
            Some(token) if self.tokens.contains(token) => Ok(req),
            _ => {
                tracing::warn!("unauthenticated request");
                Err(Status::unauthenticated("No valid auth token"))
            }
        }
    }
}
