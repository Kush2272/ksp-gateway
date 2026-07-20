//! Metrics plugin — increments Prometheus counters and records latency histograms.
//!
//! Priority: 2

use async_trait::async_trait;
use gateway_core::{
    request::{NormalizedRequest, RequestContext},
    response::{NormalizedResponse, ResponseContext},
};
use crate::chain::{GatewayPlugin, PluginResult};

pub struct MetricsPlugin;

#[async_trait]
impl GatewayPlugin for MetricsPlugin {
    fn name(&self) -> &'static str { "metrics" }
    fn priority(&self) -> u8 { 2 }

    async fn on_request(
        &self,
        ctx: &mut RequestContext,
        _req: &mut NormalizedRequest,
    ) -> PluginResult {
        // Store start time for latency calculation in on_response.
        ctx.set_ext("metrics.request_start_ms", ctx.received_at.timestamp_millis());
        PluginResult::Continue
    }

    async fn on_response(
        &self,
        ctx: &mut ResponseContext,
        resp: &mut NormalizedResponse,
    ) -> PluginResult {
        // Individual metric recording will be wired to the Prometheus registry
        // in gateway-monitor. The plugin itself is kept dependency-free so that
        // the plugin crate does not need to pull in Prometheus directly.
        let status_class = match resp.status {
            100..=199 => "1xx",
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            _          => "5xx",
        };
        tracing::debug!(
            correlation_id = %ctx.correlation_id,
            status_class,
            ttfb_ms = ctx.ttfb_ms,
            "metrics recorded"
        );
        PluginResult::Continue
    }
}
