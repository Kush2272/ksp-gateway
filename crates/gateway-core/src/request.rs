//! Normalized request representation used internally across all pipeline stages.
//!
//! `NormalizedRequest` is the in-flight representation after decoding a KSP frame
//! and before handing off to a `ProtocolAdapter`. `RequestContext` carries
//! per-request metadata that flows through the entire plugin chain.

use std::collections::HashMap;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Authority, CachePolicy, CorrelationId, PipelineKind, SessionId, StreamId};

// ─── HTTP Method ──────────────────────────────────────────────────────────────

/// HTTP method, strongly typed to avoid string comparisons on the hot path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Trace,
    Connect,
    /// Non-standard or extension methods.
    Other(String),
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get     => "GET",
            Self::Head    => "HEAD",
            Self::Post    => "POST",
            Self::Put     => "PUT",
            Self::Patch   => "PATCH",
            Self::Delete  => "DELETE",
            Self::Options => "OPTIONS",
            Self::Trace   => "TRACE",
            Self::Connect => "CONNECT",
            Self::Other(s) => s.as_str(),
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }

    pub fn is_idempotent(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Put | Self::Delete | Self::Options | Self::Trace)
    }
}

impl From<&str> for HttpMethod {
    fn from(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "GET"     => Self::Get,
            "HEAD"    => Self::Head,
            "POST"    => Self::Post,
            "PUT"     => Self::Put,
            "PATCH"   => Self::Patch,
            "DELETE"  => Self::Delete,
            "OPTIONS" => Self::Options,
            "TRACE"   => Self::Trace,
            "CONNECT" => Self::Connect,
            other     => Self::Other(other.to_owned()),
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─── Normalized Request ────────────────────────────────────────────────────────

/// A protocol-agnostic representation of an inbound request decoded from a KSP frame.
///
/// This is the canonical form passed between the KSP decoder, the plugin chain,
/// and the upstream `ProtocolAdapter`.
#[derive(Debug, Clone)]
pub struct NormalizedRequest {
    /// HTTP method (or equivalent for non-HTTP pipelines).
    pub method: HttpMethod,
    /// Full URL path + query string (e.g., `/search?q=ksp`).
    pub path_and_query: String,
    /// HTTP version hint (1 = HTTP/1.1, 2 = HTTP/2).
    pub http_version: u8,
    /// Request headers. All header names are lowercased.
    pub headers: HashMap<String, String>,
    /// Request body, if present.
    pub body: Option<Bytes>,
    /// Destination authority (host + port + TLS flag).
    pub authority: Authority,
}

impl NormalizedRequest {
    /// Returns the `Content-Length` if present and parseable.
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Returns the `Content-Type` header value, if present.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(String::as_str)
    }

    /// Returns `true` if this request's `Upgrade` header indicates a WebSocket.
    pub fn is_websocket_upgrade(&self) -> bool {
        self.headers
            .get("upgrade")
            .map(|v| v.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false)
    }
}

// ─── Request Context ──────────────────────────────────────────────────────────

/// Per-request metadata that flows through the entire plugin chain and pipeline.
///
/// Plugins may attach arbitrary key-value pairs via [`RequestContext::set_ext`]
/// and [`RequestContext::get_ext`] to pass data to downstream pipeline stages.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique correlation ID for distributed tracing.
    pub correlation_id: CorrelationId,
    /// KSP session this request arrived on.
    pub session_id: SessionId,
    /// KSP stream within that session.
    pub stream_id: StreamId,
    /// Timestamp when the request was received.
    pub received_at: DateTime<Utc>,
    /// Which pipeline will handle this request.
    pub pipeline: PipelineKind,
    /// Cache policy as determined by the router and cache plugin.
    pub cache_policy: CachePolicy,
    /// Whether the request was served from cache (set by the cache plugin).
    pub cache_hit: bool,
    /// Client IP address (the gateway's own listening socket peer).
    pub client_addr: std::net::SocketAddr,
    /// Extension map for plugin-to-plugin data passing.
    extensions: HashMap<String, serde_json::Value>,
}

impl RequestContext {
    pub fn new(
        session_id: SessionId,
        stream_id: StreamId,
        pipeline: PipelineKind,
        client_addr: std::net::SocketAddr,
    ) -> Self {
        Self {
            correlation_id: CorrelationId::new(),
            session_id,
            stream_id,
            received_at: Utc::now(),
            pipeline,
            cache_policy: CachePolicy::default(),
            cache_hit: false,
            client_addr,
            extensions: HashMap::new(),
        }
    }

    /// Store an arbitrary value for use by downstream plugins.
    pub fn set_ext(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(v) = serde_json::to_value(value) {
            self.extensions.insert(key.into(), v);
        }
    }

    /// Retrieve a value previously stored by an upstream plugin.
    pub fn get_ext<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.extensions
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Elapsed milliseconds since the request was received.
    pub fn elapsed_ms(&self) -> f64 {
        let now = Utc::now();
        let dur = now.signed_duration_since(self.received_at);
        dur.num_microseconds().unwrap_or(0) as f64 / 1000.0
    }
}
