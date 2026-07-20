//! WebSocket pipeline orchestrator — Milestone 4 full implementation.

use gateway_core::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    response::NormalizedResponse,
};

pub struct WsPipeline;

impl WsPipeline {
    pub fn new() -> Self {
        Self
    }

    /// Handle a WebSocket upgrade request.
    ///
    /// Full implementation in Milestone 4: establish WSS upstream,
    /// relay frames bidirectionally between KSP stream and WSS connection.
    pub async fn handle(
        &self,
        _ctx: RequestContext,
        _req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse> {
        Ok(NormalizedResponse::synthetic_error(
            501,
            "WebSocket pipeline not yet implemented (Milestone 4)",
        ))
    }
}

impl Default for WsPipeline {
    fn default() -> Self {
        Self::new()
    }
}
