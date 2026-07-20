//! Newtypes and enumerations used across all gateway crates.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ─── Session / Stream identifiers ────────────────────────────────────────────

/// Uniquely identifies a KSP session between the browser and the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sess-{}", &self.0.to_string()[..8])
    }
}

/// Identifies a multiplexed stream within a KSP session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId(pub u32);

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stream-{}", self.0)
    }
}

/// A per-request correlation identifier for distributed tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Pipeline classification ──────────────────────────────────────────────────

/// Describes which protocol pipeline should handle a given request.
///
/// The router's `RoutePlanner` returns one of these variants after inspecting
/// the first frames of an incoming KSP stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineKind {
    /// Standard HTTP/1.1 or HTTP/2 request/response.
    Http,
    /// WebSocket upgrade from HTTP.
    WebSocket,
    /// Native KSP-to-KSP pass-through (no translation needed).
    NativeKsp,
    /// Future: FTP protocol bridge.
    Ftp,
    /// Future: SSH protocol bridge.
    Ssh,
    /// Future: gRPC protocol bridge.
    Grpc,
}

impl fmt::Display for PipelineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http      => write!(f, "HTTP"),
            Self::WebSocket => write!(f, "WebSocket"),
            Self::NativeKsp => write!(f, "NativeKSP"),
            Self::Ftp       => write!(f, "FTP"),
            Self::Ssh       => write!(f, "SSH"),
            Self::Grpc      => write!(f, "gRPC"),
        }
    }
}

// ─── Authority (host + optional port) ────────────────────────────────────────

/// A normalized `host[:port]` pair identifying an upstream origin.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Authority {
    pub host: String,
    pub port: u16,
    pub tls:  bool,
}

impl Authority {
    pub fn new(host: impl Into<String>, port: u16, tls: bool) -> Self {
        Self { host: host.into(), port, tls }
    }

    /// Default HTTPS origin (port 443, TLS on).
    pub fn https(host: impl Into<String>) -> Self {
        Self::new(host, 443, true)
    }

    /// Default HTTP origin (port 80, TLS off).
    pub fn http(host: impl Into<String>) -> Self {
        Self::new(host, 80, false)
    }
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

// ─── Cache policy ─────────────────────────────────────────────────────────────

/// Controls whether and how a response may be cached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Always fetch from origin, never store.
    NoStore,
    /// Always fetch from origin, but store.
    NoCache,
    /// Cache for the given number of seconds.
    MaxAge(u32),
    /// Response must be revalidated with origin after `max_age` seconds.
    MustRevalidate { max_age: u32 },
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self::NoStore
    }
}
