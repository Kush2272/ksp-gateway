//! Protocol detector + pipeline router.
//!
//! Inspects a decoded request and dispatches it to the correct pipeline.

use std::sync::Arc;
use gateway_core::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    response::NormalizedResponse,
};
use gateway_pipeline_http::HttpPipeline;
use gateway_pipeline_ws::WsPipeline;

pub struct PipelineRouter {
    http: Arc<HttpPipeline>,
    ws:   Arc<WsPipeline>,
}

impl PipelineRouter {
    pub fn new(http: Arc<HttpPipeline>, ws: Arc<WsPipeline>) -> Self {
        Self { http, ws }
    }

    /// Detect the correct pipeline from the request and dispatch.
    pub async fn dispatch(
        &self,
        ctx: RequestContext,
        req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse> {
        if req.is_websocket_upgrade() {
            self.ws.handle(ctx, req).await
        } else {
            self.http.handle(ctx, req).await
        }
    }
}
