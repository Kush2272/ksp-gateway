//! Tracing subscriber initialisation.

use gateway_config::schema::MonitorConfig;

/// Initialise the global tracing subscriber based on the monitor configuration.
///
/// Call this exactly once at startup from `gateway-cli`.
pub fn init_tracing(cfg: &MonitorConfig) {
    use tracing_subscriber::{fmt, EnvFilter, prelude::*};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.log_level));

    match cfg.log_format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().pretty())
                .init();
        }
    }
}
