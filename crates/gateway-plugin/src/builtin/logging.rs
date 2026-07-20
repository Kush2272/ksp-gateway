//! Logging plugin — emits a structured log line for every request and response.
//!
//! Priority: 1 (runs first so all downstream plugins benefit from the correlation ID
//! already being set in the tracing span).

use async_trait::async_trait;
use gateway_core::{
    request::{NormalizedRequest, RequestContext},
    response::{NormalizedResponse, ResponseContext},
};
use crate::chain::{GatewayPlugin, PluginResult};

pub struct LoggingPlugin;

#[async_trait]
impl GatewayPlugin for LoggingPlugin {
    fn name(&self) -> &'static str { "logging" }
    fn priority(&self) -> u8 { 1 }

    async fn on_request(
        &self,
        ctx: &mut RequestContext,
        req: &mut NormalizedRequest,
    ) -> PluginResult {
        tracing::info!(
            correlation_id = %ctx.correlation_id,
            session_id     = %ctx.session_id,
            stream_id      = %ctx.stream_id,
            method         = %req.method,
            authority      = %req.authority,
            path           = %req.path_and_query,
            pipeline       = %ctx.pipeline,
            "→ Request"
        );
        PluginResult::Continue
    }

    async fn on_response(
        &self,
        ctx: &mut ResponseContext,
        resp: &mut NormalizedResponse,
    ) -> PluginResult {
        tracing::info!(
            correlation_id = %ctx.correlation_id,
            status         = resp.status,
            cache_hit      = ctx.cache_hit,
            ttfb_ms        = ctx.ttfb_ms,
            "← Response"
        );
        PluginResult::Continue
    }
}
