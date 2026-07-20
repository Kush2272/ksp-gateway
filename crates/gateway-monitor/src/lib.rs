//! # gateway-monitor
//!
//! Prometheus metrics registry, structured tracing initialisation, and
//! health-check endpoint for KSP Gateway.

pub mod metrics;
pub mod tracing_init;
pub mod health;

pub use metrics::GatewayMetrics;
pub use tracing_init::init_tracing;
pub use health::HealthStatus;
