//! Config file loader with environment variable overrides.

use std::path::Path;
use gateway_core::error::{GatewayError, GatewayResult};
use crate::schema::GatewayConfig;

/// Load a `GatewayConfig` from a TOML file.
///
/// If `path` does not exist, the default configuration is returned and a
/// warning is logged. This makes it trivial to run the gateway with no
/// config file during development.
pub fn load_config(path: &Path) -> GatewayResult<GatewayConfig> {
    if !path.exists() {
        tracing::warn!(
            path = %path.display(),
            "Config file not found — using defaults"
        );
        let cfg = GatewayConfig::default();
        cfg.validate().map_err(GatewayError::Config)?;
        return Ok(cfg);
    }

    let raw = std::fs::read_to_string(path)
        .map_err(|e| GatewayError::Config(format!("Failed to read {}: {e}", path.display())))?;

    let cfg: GatewayConfig = toml::from_str(&raw)
        .map_err(|e| GatewayError::Config(format!("TOML parse error in {}: {e}", path.display())))?;

    cfg.validate().map_err(GatewayError::Config)?;

    tracing::info!(path = %path.display(), "Configuration loaded");
    Ok(cfg)
}
