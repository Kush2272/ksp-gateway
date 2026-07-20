//! # gateway-plugin
//!
//! Defines the `GatewayPlugin` trait, the `PluginChain` executor, and the
//! built-in first-party plugins that ship with KSP Gateway.
//!
//! ## Plugin Lifecycle
//!
//! For every request, the chain executes `on_request` for each plugin in
//! ascending priority order. If any plugin returns `PluginResult::ShortCircuit`,
//! the chain stops and the enclosed response is returned directly to the
//! browser (e.g., a cache hit). If `PluginResult::Abort` is returned, the
//! gateway sends an error frame.
//!
//! After the upstream adapter returns a response, the chain executes
//! `on_response` for each plugin in the same order.

pub mod chain;
pub mod registry;
pub mod builtin;

pub use chain::{PluginChain, PluginResult};
pub use registry::PluginRegistry;
