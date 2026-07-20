//! # gateway-router
//!
//! Implements the Route Planner → Connection Manager → Protocol Adapter chain.
//!
//! ## Routing Flow
//!
//! ```text
//! NormalizedRequest
//!     │
//!     ▼
//! RoutePlanner::plan()   → RouteDecision { pipeline, target, cache_policy, verify_tls }
//!     │
//!     ▼
//! ConnectionManager::acquire()  → Box<dyn ProtocolAdapter>
//!     │
//!     ▼
//! ProtocolAdapter::send()  → NormalizedResponse
//! ```

pub mod planner;
pub mod connection;
pub mod adapter;
pub mod tls;
pub mod pool;

pub use planner::DefaultRoutePlanner;
pub use connection::GatewayConnectionManager;
