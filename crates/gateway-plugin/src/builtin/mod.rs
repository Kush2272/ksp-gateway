//! Built-in plugin implementations.
//!
//! These are the first-party plugins that ship with every KSP Gateway installation.
//! They are registered automatically by the CLI on startup.

pub mod logging;
pub mod metrics;
pub mod headers;

pub use logging::LoggingPlugin;
pub use metrics::MetricsPlugin;
pub use headers::HeadersPlugin;
