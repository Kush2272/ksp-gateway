//! Route Planner — analyses a normalized request and returns a `RouteDecision`.

use std::sync::Arc;
use async_trait::async_trait;
use tracing::debug;

use gateway_core::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    traits::{RouteDecision, RoutePlanner},
    types::{CachePolicy, PipelineKind},
};
use gateway_config::schema::RouterConfig;
use gateway_resolver::GatewayResolver;

#[allow(dead_code)]
pub struct DefaultRoutePlanner {
    resolver: Arc<GatewayResolver>,
    config:   Arc<RouterConfig>,
}

impl DefaultRoutePlanner {
    pub fn new(resolver: Arc<GatewayResolver>, config: Arc<RouterConfig>) -> Self {
        Self { resolver, config }
    }
}

#[async_trait]
impl RoutePlanner for DefaultRoutePlanner {
    async fn plan(
        &self,
        req: &NormalizedRequest,
        _ctx: &RequestContext,
    ) -> GatewayResult<RouteDecision> {
        // Determine which pipeline to use.
        let pipeline = if req.is_websocket_upgrade() {
            PipelineKind::WebSocket
        } else {
            PipelineKind::Http
        };

        // Determine cache policy: non-safe methods bypass cache.
        let cache_policy = if req.method.is_safe() {
            CachePolicy::MaxAge(300)
        } else {
            CachePolicy::NoStore
        };

        debug!(
            host = %req.authority.host,
            pipeline = %pipeline,
            "Route planned"
        );

        Ok(RouteDecision {
            pipeline,
            target:     req.authority.clone(),
            cache_policy,
            verify_tls: true,
        })
    }
}
