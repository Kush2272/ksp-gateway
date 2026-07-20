//! # gateway-pipeline-ws
//!
//! WebSocket pipeline for KSP Gateway.
//!
//! ## Frame relay flow (Milestone 4)
//!
//! ```text
//! KSP Stream (browser)
//!     │  WebSocket frames encoded in KSP DATA packets
//!     ▼
//! WsPipeline
//!     ├── Negotiate HTTP Upgrade with origin
//!     ├── Establish WSS connection
//!     └── Bidirectional relay:
//!             KSP ←→ WSS frames
//! ```

pub mod pipeline;
pub mod frame;

pub use pipeline::WsPipeline;
