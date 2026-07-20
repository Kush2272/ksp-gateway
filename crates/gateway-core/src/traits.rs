//! Core traits that define the interfaces between all gateway components.
//!
//! Every major gateway component is defined as a trait here so that:
//! - Components can be tested in isolation with mock implementations.
//! - The plugin system can compose components uniformly.
//! - Future protocol adapters can be added without changing callers.

use std::net::IpAddr;
use async_trait::async_trait;

use crate::{
    error::GatewayResult,
    request::{NormalizedRequest, RequestContext},
    response::NormalizedResponse,
    types::{Authority, CachePolicy, PipelineKind},
};

// ─── Resolver ─────────────────────────────────────────────────────────────────

/// Resolves a hostname to one or more IP addresses.
///
/// The gateway uses a `Resolver` before handing off to the `RoutePlanner`.
/// Implementations include the system resolver, DNS-over-HTTPS, and a
/// custom-hosts override layer.
#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    /// Resolve `host` to a list of IP addresses. Returns at least one address
    /// on success, or a `GatewayError::DnsResolution` on failure.
    async fn resolve(&self, host: &str) -> GatewayResult<Vec<IpAddr>>;
}

// ─── Route Planner ────────────────────────────────────────────────────────────

/// Decides which pipeline should handle a request and determines the origin target.
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// Which pipeline will handle this stream.
    pub pipeline: PipelineKind,
    /// Resolved origin target.
    pub target: Authority,
    /// Cache policy as determined by host/path rules.
    pub cache_policy: CachePolicy,
    /// Whether upstream TLS certificate verification is required.
    pub verify_tls: bool,
}

#[async_trait]
pub trait RoutePlanner: Send + Sync + 'static {
    /// Analyse the incoming request and return a routing decision.
    async fn plan(
        &self,
        req: &NormalizedRequest,
        ctx: &RequestContext,
    ) -> GatewayResult<RouteDecision>;
}

// ─── Connection Manager ───────────────────────────────────────────────────────

/// Acquires (or creates) an upstream connection for a given route decision.
///
/// Implementations manage per-origin connection pools and TLS session reuse.
#[async_trait]
pub trait ConnectionManager: Send + Sync + 'static {
    type Connection: ProtocolAdapter;

    /// Acquire a connection to the origin described by `decision`.
    async fn acquire(&self, decision: &RouteDecision) -> GatewayResult<Self::Connection>;

    /// Return a connection to the pool after use (may be called implicitly on drop).
    async fn release(&self, conn: Self::Connection);
}

// ─── Protocol Adapter ─────────────────────────────────────────────────────────

/// Speaks to an upstream origin in its native protocol (HTTP/1.1, HTTP/2, etc.).
///
/// The adapter's job is purely mechanical: take a `NormalizedRequest`, produce
/// a `NormalizedResponse`. All cross-cutting concerns (caching, auth, headers)
/// are handled by the plugin chain before and after the adapter is called.
#[async_trait]
pub trait ProtocolAdapter: Send + 'static {
    /// Send the request to the origin and return the full response.
    async fn send(
        &mut self,
        req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse>;

    /// Returns the HTTP version this connection is using (1 or 2).
    fn http_version(&self) -> u8;
}

// ─── Cache Store ──────────────────────────────────────────────────────────────

/// A cache key derived from a normalized request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub method: String,
    pub authority: String,
    pub path_and_query: String,
    pub vary_headers: Vec<(String, String)>,
}

impl CacheKey {
    pub fn from_request(req: &NormalizedRequest) -> Self {
        Self {
            method: req.method.as_str().to_owned(),
            authority: req.authority.to_string(),
            path_and_query: req.path_and_query.clone(),
            vary_headers: Vec::new(), // populated after cache plugin inspects Vary header
        }
    }
}

#[async_trait]
pub trait CacheStore: Send + Sync + 'static {
    /// Look up a cached response. Returns `None` on miss.
    async fn get(&self, key: &CacheKey) -> Option<NormalizedResponse>;

    /// Store a response. `ttl_secs` is how long to keep it.
    async fn put(&self, key: CacheKey, resp: NormalizedResponse, ttl_secs: u32);

    /// Invalidate a single cache entry.
    async fn invalidate(&self, key: &CacheKey);

    /// Invalidate all entries for a given authority (used after non-safe methods).
    async fn invalidate_authority(&self, authority: &str);

    /// Return approximate cache size in bytes.
    async fn size_bytes(&self) -> u64;

    /// Return hit/miss counts since last reset.
    async fn stats(&self) -> CacheStats;
}

/// Cache hit/miss statistics.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CacheStats {
    pub hits:   u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

// ─── Metrics Provider ─────────────────────────────────────────────────────────

/// An abstraction over the metrics backend (Prometheus, StatsD, etc.).
///
/// Plugins use this to record observations without a hard dependency on Prometheus.
pub trait MetricsProvider: Send + Sync + 'static {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);
    fn observe_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn set_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

// ─── Session Handler ─────────────────────────────────────────────────────────

/// Lifecycle callbacks for KSP sessions.
///
/// Implemented by the session manager in `gateway-ksp` and called by the
/// top-level listener when sessions are created, active, or terminated.
#[async_trait]
pub trait SessionHandler: Send + Sync + 'static {
    /// Called when a new KSP session is fully established.
    async fn on_connect(
        &self,
        session_id: crate::types::SessionId,
        peer_addr: std::net::SocketAddr,
    ) -> GatewayResult<()>;

    /// Called for each decoded request frame ready for routing.
    async fn on_request(
        &self,
        ctx: RequestContext,
        req: NormalizedRequest,
    ) -> GatewayResult<NormalizedResponse>;

    /// Called when a session is gracefully or forcefully closed.
    async fn on_disconnect(&self, session_id: crate::types::SessionId);
}
