//! # gateway-pipeline-http
//!
//! HTTP/1.1 and HTTP/2 pipeline for KSP Gateway.
//!
//! ## Pipeline Stages (Milestone 3)
//!
//! ```text
//! NormalizedRequest
//!     │
//!     ├── [Plugin Chain: request side]
//!     │       dns, cache, headers, compression, auth, rate-limit
//!     │
//!     ├── RoutePlanner::plan()
//!     │
//!     ├── ConnectionManager::acquire()
//!     │
//!     ├── ProtocolAdapter::send()  (HTTP/1.1 or HTTP/2)
//!     │
//!     ├── [Plugin Chain: response side]
//!     │       cache store, compression decode, header cleanup
//!     │
//!     └── NormalizedResponse → KSP encoder
//! ```
//!
//! The pipeline is independent of the WebSocket pipeline; adding future
//! pipelines (gRPC, FTP) requires no changes here.

pub mod pipeline;
pub mod redirect;
pub mod compression;

pub use pipeline::HttpPipeline;
