//! # gateway-config
//!
//! Configuration loading, validation, and live-reload for KSP Gateway.
//! All other crates read config through the `GatewayConfig` struct.

pub mod loader;
pub mod schema;
pub mod watcher;

pub use schema::GatewayConfig;
pub use loader::load_config;
