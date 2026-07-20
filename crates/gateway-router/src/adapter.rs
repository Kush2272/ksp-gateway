//! Protocol adapter skeleton — Milestone 1 placeholder that will be fully
//! implemented in Milestone 3 (gateway-pipeline-http).

use gateway_core::{
    error::GatewayResult,
    request::NormalizedRequest,
    response::NormalizedResponse,
};

/// Adapter that will make the actual HTTP request.
/// Full implementation lives in `gateway-pipeline-http`.
pub struct HttpAdapter;

impl HttpAdapter {
    pub async fn send(&mut self, _req: NormalizedRequest) -> GatewayResult<NormalizedResponse> {
        // Milestone 3: replace with actual hyper request.
        Ok(NormalizedResponse::synthetic_error(501, "HTTP pipeline not yet implemented"))
    }
}
