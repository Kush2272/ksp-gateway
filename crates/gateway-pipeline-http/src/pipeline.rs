//! HTTP pipeline orchestrator.
//!
//! `HttpPipeline::handle` is the single entry point for all HTTP requests.
//! It runs the plugin chain, calls the router, delegates to the connection
//! manager, and streams the response back.

use std::sync::Arc;
use gateway_core::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    response::NormalizedResponse,
    traits::RoutePlanner,
};
use gateway_plugin::PluginChain;
use gateway_router::{DefaultRoutePlanner, GatewayConnectionManager};

pub struct HttpPipeline {
    plugin_chain: Arc<PluginChain>,
    planner:      Arc<DefaultRoutePlanner>,
    conn_manager: Arc<GatewayConnectionManager>,
}

impl HttpPipeline {
    pub fn new(
        plugin_chain: Arc<PluginChain>,
        planner:      Arc<DefaultRoutePlanner>,
        conn_manager: Arc<GatewayConnectionManager>,
    ) -> Self {
        Self { plugin_chain, planner, conn_manager }
    }

    /// Handle a single HTTP request through the full pipeline.
    pub async fn handle(
        &self,
        mut ctx: RequestContext,
        mut req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse> {
        // ── Request-side plugin chain ────────────────────────────────────────
        if let Some(cached_resp) = self.plugin_chain.run_request(&mut ctx, &mut req).await? {
            return Ok(cached_resp);
        }

        // ── Route planning ───────────────────────────────────────────────────
        let decision = self.planner.plan(&req, &ctx).await?;

        // ── Connection acquisition ───────────────────────────────────────────
        let _conn = self.conn_manager.acquire(&decision).await?;

        // ── Protocol adapter (Milestone 3: full HTTP/hyper implementation) ───
        let mut resp = NormalizedResponse::synthetic_error(
            501,
            "HTTP pipeline adapter not yet implemented (Milestone 3)",
        );

        // ── Response-side plugin chain ───────────────────────────────────────
        use gateway_core::response::ResponseContext;
        let mut resp_ctx = ResponseContext::new(ctx.correlation_id, resp.status, 0.0);
        self.plugin_chain.run_response(&mut resp_ctx, &mut resp).await?;

        Ok(resp)
    }
}
