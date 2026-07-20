//! WebSocket pipeline orchestrator — Milestone 4 implementation.

use std::collections::HashMap;
use tokio_tungstenite::connect_async;
use tracing::info;

use gateway_core::{
    error::{GatewayError, GatewayResult},
    request::{NormalizedRequest, RequestContext},
    response::NormalizedResponse,
};

pub struct WsPipeline;

impl WsPipeline {
    pub fn new() -> Self {
        Self
    }

    /// Handle a WebSocket upgrade request by establishing an upstream WebSocket connection.
    pub async fn handle(
        &self,
        _ctx: RequestContext,
        req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse> {
        let scheme = if req.authority.tls { "wss" } else { "ws" };
        let url = format!("{}://{}{}", scheme, req.authority.to_string(), req.path_and_query);

        info!(target_url = %url, "Establishing upstream WebSocket connection");

        // Attempt WebSocket handshake with upstream
        let (_ws_stream, response) = connect_async(&url)
            .await
            .map_err(|e| GatewayError::WsUpgrade(format!("Failed to connect to WS upstream {url}: {e}")))?;

        let status = response.status().as_u16();
        let reason = response.status().canonical_reason().unwrap_or("Switching Protocols").to_string();

        let mut headers = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.insert(k.as_str().to_lowercase(), v_str.to_string());
            }
        }

        Ok(NormalizedResponse {
            status,
            reason,
            http_version: 1,
            headers,
            body: None,
            is_streaming: true,
        })
    }
}

impl Default for WsPipeline {
    fn default() -> Self {
        Self::new()
    }
}
