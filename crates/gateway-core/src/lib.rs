//! # gateway-core
//!
//! Shared types, error definitions, newtypes, and core traits for the KSP Gateway.
//! Every other gateway crate depends on this crate; it has no internal gateway dependencies.

pub mod error;
pub mod types;
pub mod request;
pub mod response;
pub mod traits;

pub use error::{GatewayError, GatewayResult};
pub use types::{SessionId, StreamId, CorrelationId, PipelineKind, Authority};
pub use request::{NormalizedRequest, RequestContext};
pub use response::{NormalizedResponse, ResponseContext};
