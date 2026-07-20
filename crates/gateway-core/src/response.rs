//! Normalized response representation used internally across all pipeline stages.

use std::collections::HashMap;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{CachePolicy, CorrelationId};

// ─── Normalized Response ──────────────────────────────────────────────────────

/// A protocol-agnostic representation of a response from an upstream origin.
///
/// This is produced by a `ProtocolAdapter` and consumed by the response-side
/// plugin chain before being encoded into KSP frames and sent to the browser.
#[derive(Debug, Clone)]
pub struct NormalizedResponse {
    /// HTTP status code.
    pub status: u16,
    /// HTTP reason phrase.
    pub reason: String,
    /// HTTP version the upstream used (1 = HTTP/1.1, 2 = HTTP/2).
    pub http_version: u8,
    /// Response headers. All names are lowercased.
    pub headers: HashMap<String, String>,
    /// Fully-buffered body, if available.
    /// For large/streaming responses this will be `None`; use the streaming API.
    pub body: Option<Bytes>,
    /// Whether this response was delivered as chunked/streaming.
    pub is_streaming: bool,
}

impl NormalizedResponse {
    /// Constructs a minimal synthetic error response (not from upstream).
    pub fn synthetic_error(status: u16, message: impl Into<String>) -> Self {
        let body = message.into().into_bytes();
        let len  = body.len();
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/plain; charset=utf-8".into());
        headers.insert("content-length".into(), len.to_string());
        headers.insert("x-ksp-synthetic".into(), "true".into());
        Self {
            status,
            reason: http_reason(status).to_owned(),
            http_version: 1,
            headers,
            body: Some(Bytes::from(body)),
            is_streaming: false,
        }
    }

    /// Returns `true` if this is a redirect response.
    pub fn is_redirect(&self) -> bool {
        matches!(self.status, 301 | 302 | 303 | 307 | 308)
    }

    /// Returns the `Location` header value for redirect responses.
    pub fn location(&self) -> Option<&str> {
        self.headers.get("location").map(String::as_str)
    }

    /// Returns the `Content-Type` header value, if present.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(String::as_str)
    }

    /// Returns the `Content-Length` if present and parseable.
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Returns `true` if the response is considered cacheable by default rules.
    pub fn is_cacheable(&self) -> bool {
        matches!(self.status, 200 | 203 | 204 | 206 | 300 | 301 | 404 | 405 | 410 | 414 | 501)
    }
}

// ─── Response Context ──────────────────────────────────────────────────────────

/// Per-response metadata that flows through the response-side plugin chain.
#[derive(Debug, Clone)]
pub struct ResponseContext {
    /// Correlation ID matching the originating `RequestContext`.
    pub correlation_id: CorrelationId,
    /// HTTP status code (mirrors `NormalizedResponse::status`).
    pub status: u16,
    /// Whether this response is served from the cache.
    pub cache_hit: bool,
    /// Resolved cache policy for storing this response.
    pub cache_policy: CachePolicy,
    /// Timestamp when the response was received from the origin.
    pub received_at: DateTime<Utc>,
    /// Latency from request dispatch to first byte of response (ms).
    pub ttfb_ms: f64,
    /// Extension map for plugin-to-plugin data passing.
    extensions: HashMap<String, serde_json::Value>,
}

impl ResponseContext {
    pub fn new(correlation_id: CorrelationId, status: u16, ttfb_ms: f64) -> Self {
        Self {
            correlation_id,
            status,
            cache_hit: false,
            cache_policy: CachePolicy::default(),
            received_at: Utc::now(),
            ttfb_ms,
            extensions: HashMap::new(),
        }
    }

    pub fn set_ext(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(v) = serde_json::to_value(value) {
            self.extensions.insert(key.into(), v);
        }
    }

    pub fn get_ext<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.extensions
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn http_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _   => "Unknown",
    }
}
