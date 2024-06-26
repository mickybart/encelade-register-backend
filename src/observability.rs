//! Observability module to control logging

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging
///
/// Based on RUST_LOG environment variable
///
/// ```bash
/// export RUST_LOG="encelade_register_backend=debug,info"
/// ```
pub(crate) fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new("encelade_register_backend=debug,info"));

    let logs_layer = tracing_subscriber::fmt::layer();
    // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(logs_layer)
        .init();
}
