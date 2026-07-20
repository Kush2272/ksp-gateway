//! Plugin trait definition and chain executor.

use std::sync::Arc;
use async_trait::async_trait;
use gateway_core::{
    error::GatewayError,
    request::{NormalizedRequest, RequestContext},
    response::{NormalizedResponse, ResponseContext},
};

// ─── Plugin Result ─────────────────────────────────────────────────────────────

/// The result returned by each plugin hook.
pub enum PluginResult {
    /// Continue to the next plugin in the chain.
    Continue,
    /// Stop the chain and return this response directly (e.g., cache hit).
    ShortCircuit(NormalizedResponse),
    /// Abort the request with an error.
    Abort(GatewayError),
}

// ─── GatewayPlugin trait ──────────────────────────────────────────────────────

/// The core extension point for KSP Gateway.
///
/// Every built-in feature (caching, compression, logging, auth, rate-limiting)
/// is implemented as a `GatewayPlugin`. Third-party plugins in the future will
/// implement this same trait via WASM/Lua runtimes.
///
/// # Priority
///
/// Plugins are sorted by [`GatewayPlugin::priority`] in ascending order before
/// the chain is built. Lower numbers run first. The recommended ranges are:
///
/// - `0–9`:   Infrastructure (logging, metrics, correlation IDs)
/// - `10–49`: Security (rate-limiting, authentication, host blocking)
/// - `50–89`: Transformation (headers, compression, cookies)
/// - `90–99`: Caching
#[async_trait]
pub trait GatewayPlugin: Send + Sync + 'static {
    /// Unique, machine-readable plugin name (e.g., `"cache"`, `"compression"`).
    fn name(&self) -> &'static str;

    /// Execution priority. Lower values run first.
    fn priority(&self) -> u8;

    /// Called once per request before the upstream adapter is invoked.
    ///
    /// Plugins may mutate `ctx` and `req` in place, or return a
    /// `PluginResult::ShortCircuit` to bypass the origin entirely.
    async fn on_request(
        &self,
        ctx: &mut RequestContext,
        req: &mut NormalizedRequest,
    ) -> PluginResult;

    /// Called once per response after the upstream adapter returns.
    ///
    /// Plugins may mutate `ctx` and `resp` in place (e.g., to add headers,
    /// compress the body, or store the response in cache).
    async fn on_response(
        &self,
        ctx: &mut ResponseContext,
        resp: &mut NormalizedResponse,
    ) -> PluginResult;
}

// ─── Plugin Chain ─────────────────────────────────────────────────────────────

/// An ordered list of plugins executed for every request/response pair.
///
/// The chain is immutable after construction; to change the plugin set
/// you rebuild the chain (typically at config reload time).
pub struct PluginChain {
    plugins: Vec<Arc<dyn GatewayPlugin>>,
}

impl PluginChain {
    /// Build a `PluginChain` from a list of plugins, sorted by priority.
    pub fn new(mut plugins: Vec<Arc<dyn GatewayPlugin>>) -> Self {
        plugins.sort_by_key(|p| p.priority());
        Self { plugins }
    }

    /// Return the number of plugins in the chain.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Run the request-side of the chain.
    ///
    /// Returns `Ok(None)` if all plugins returned `Continue` (proceed to adapter).
    /// Returns `Ok(Some(resp))` if a plugin short-circuited with a cached response.
    /// Returns `Err(e)` if a plugin aborted the request.
    pub async fn run_request(
        &self,
        ctx: &mut RequestContext,
        req: &mut NormalizedRequest,
    ) -> Result<Option<NormalizedResponse>, GatewayError> {
        for plugin in &self.plugins {
            let span = tracing::debug_span!(
                "plugin.request",
                plugin = plugin.name(),
                priority = plugin.priority(),
            );
            let _enter = span.enter();

            match plugin.on_request(ctx, req).await {
                PluginResult::Continue => continue,
                PluginResult::ShortCircuit(resp) => {
                    tracing::debug!(
                        plugin = plugin.name(),
                        status = resp.status,
                        "Plugin short-circuited request"
                    );
                    return Ok(Some(resp));
                }
                PluginResult::Abort(e) => {
                    tracing::warn!(
                        plugin = plugin.name(),
                        error = %e,
                        "Plugin aborted request"
                    );
                    return Err(e);
                }
            }
        }
        Ok(None)
    }

    /// Run the response-side of the chain.
    ///
    /// Returns `Ok(())` if all plugins returned `Continue`.
    /// Returns `Err(e)` if a plugin aborted the response.
    pub async fn run_response(
        &self,
        ctx: &mut ResponseContext,
        resp: &mut NormalizedResponse,
    ) -> Result<(), GatewayError> {
        for plugin in &self.plugins {
            let span = tracing::debug_span!(
                "plugin.response",
                plugin = plugin.name(),
                priority = plugin.priority(),
                status = resp.status,
            );
            let _enter = span.enter();

            match plugin.on_response(ctx, resp).await {
                PluginResult::Continue => continue,
                PluginResult::ShortCircuit(_) => {
                    // Short-circuiting on the response side is unusual but valid
                    // (e.g., a security plugin replaces a 200 with a 403).
                    tracing::warn!(
                        plugin = plugin.name(),
                        "Plugin short-circuited response side (unexpected)"
                    );
                    break;
                }
                PluginResult::Abort(e) => {
                    tracing::warn!(
                        plugin = plugin.name(),
                        error = %e,
                        "Plugin aborted response"
                    );
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}
