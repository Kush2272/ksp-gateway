//! Headers plugin — normalizes, strips hop-by-hop headers, and injects
//! forwarding headers on both request and response sides.
//!
//! Priority: 50

use async_trait::async_trait;
use gateway_core::{
    request::{NormalizedRequest, RequestContext},
    response::{NormalizedResponse, ResponseContext},
};
use crate::chain::{GatewayPlugin, PluginResult};

/// Headers that must never be forwarded to the upstream origin.
/// These are "hop-by-hop" headers defined in RFC 7230 §6.1.
static HOP_BY_HOP_REQUEST: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

/// Headers that must never be forwarded back to the browser from the upstream.
static HOP_BY_HOP_RESPONSE: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
];

pub struct HeadersPlugin;

#[async_trait]
impl GatewayPlugin for HeadersPlugin {
    fn name(&self) -> &'static str { "headers" }
    fn priority(&self) -> u8 { 50 }

    async fn on_request(
        &self,
        ctx: &mut RequestContext,
        req: &mut NormalizedRequest,
    ) -> PluginResult {
        // Remove hop-by-hop headers.
        for name in HOP_BY_HOP_REQUEST {
            req.headers.remove(*name);
        }

        // Inject forwarding headers.
        req.headers.insert(
            "x-forwarded-for".into(),
            ctx.client_addr.ip().to_string(),
        );
        req.headers.insert(
            "x-forwarded-proto".into(),
            if req.authority.tls { "https".into() } else { "http".into() },
        );
        req.headers.insert(
            "x-ksp-correlation-id".into(),
            ctx.correlation_id.to_string(),
        );

        // Ensure Host header is set correctly.
        req.headers.insert("host".into(), req.authority.host.clone());

        PluginResult::Continue
    }

    async fn on_response(
        &self,
        _ctx: &mut ResponseContext,
        resp: &mut NormalizedResponse,
    ) -> PluginResult {
        // Remove hop-by-hop headers from the upstream response.
        for name in HOP_BY_HOP_RESPONSE {
            resp.headers.remove(*name);
        }

        // Inject gateway identifier header.
        resp.headers.insert(
            "x-ksp-gateway".into(),
            "ksp-gateway/0.1.0".into(),
        );

        PluginResult::Continue
    }
}
