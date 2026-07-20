//! HTTP pipeline orchestrator.
//!
//! `HttpPipeline::handle` is the single entry point for all HTTP requests.
//! It runs the plugin chain, calls the router, delegates to the connection
//! manager, and streams the response back.

use std::sync::Arc;
use gateway_core::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    response::{NormalizedResponse, ResponseContext},
    traits::RoutePlanner,
};
use gateway_plugin::PluginChain;
use gateway_router::{adapter::HttpAdapter, DefaultRoutePlanner, GatewayConnectionManager};

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
        let conn = self.conn_manager.acquire(&decision).await?;

        // ── Protocol adapter ─────────────────────────────────────────────────
        let start_time = std::time::Instant::now();
        let mut adapter = HttpAdapter::new(conn);
        let mut resp = adapter.send(req).await?;
        let ttfb_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        // ── Response-side plugin chain ───────────────────────────────────────
        let mut resp_ctx = ResponseContext::new(ctx.correlation_id, resp.status, ttfb_ms);
        self.plugin_chain.run_response(&mut resp_ctx, &mut resp).await?;

        Ok(resp)
    }
}
