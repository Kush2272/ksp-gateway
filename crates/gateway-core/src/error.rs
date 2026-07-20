//! Error types for KSP Gateway.
//!
//! All gateway crates return [`GatewayError`] variants, allowing a unified
//! error handling story at the top-level CLI and session handler.

use thiserror::Error;

/// The top-level error type for all gateway operations.
#[derive(Debug, Error)]
pub enum GatewayError {
    // ── KSP layer errors ────────────────────────────────────────────────────
    #[error("KSP session error: {0}")]
    KspSession(String),

    #[error("KSP frame decode error: {0}")]
    KspFrameDecode(String),

    #[error("KSP frame encode error: {0}")]
    KspFrameEncode(String),

    #[error("KSP handshake failed: {0}")]
    KspHandshake(String),

    // ── Resolver errors ──────────────────────────────────────────────────────
    #[error("DNS resolution failed for '{host}': {reason}")]
    DnsResolution { host: String, reason: String },

    #[error("Host is not allowed: {0}")]
    HostBlocked(String),

    // ── Router / pipeline errors ─────────────────────────────────────────────
    #[error("No pipeline available for protocol kind: {0:?}")]
    NoPipeline(crate::types::PipelineKind),

    #[error("Route planning failed: {0}")]
    RoutePlan(String),

    // ── Connection errors ────────────────────────────────────────────────────
    #[error("Connection pool exhausted for origin '{0}'")]
    PoolExhausted(String),

    #[error("Connection to origin failed: {0}")]
    Connect(String),

    #[error("TLS handshake failed: {0}")]
    Tls(String),

    // ── HTTP errors ──────────────────────────────────────────────────────────
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Too many redirects (max {max}, seen {seen})")]
    TooManyRedirects { max: u8, seen: u8 },

    #[error("Upstream returned status {0}")]
    UpstreamStatus(u16),

    // ── WebSocket errors ─────────────────────────────────────────────────────
    #[error("WebSocket upgrade failed: {0}")]
    WsUpgrade(String),

    #[error("WebSocket frame error: {0}")]
    WsFrame(String),

    // ── Plugin errors ────────────────────────────────────────────────────────
    #[error("Plugin '{name}' error: {reason}")]
    Plugin { name: String, reason: String },

    // ── Cache errors ─────────────────────────────────────────────────────────
    #[error("Cache error: {0}")]
    Cache(String),

    // ── Config errors ────────────────────────────────────────────────────────
    #[error("Configuration error: {0}")]
    Config(String),

    // ── I/O and generic errors ───────────────────────────────────────────────
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Rate limit exceeded for session {0}")]
    RateLimit(String),

    #[error("Request payload too large: {size} bytes (max {max})")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Internal gateway error: {0}")]
    Internal(String),
}

/// Convenience alias.
pub type GatewayResult<T> = Result<T, GatewayError>;

impl GatewayError {
    /// Returns an appropriate HTTP status code hint for this error,
    /// used when translating errors back to KSP error frames.
    pub fn http_status_hint(&self) -> u16 {
        match self {
            Self::DnsResolution { .. }     => 502,
            Self::HostBlocked(_)           => 403,
            Self::NoPipeline(_)            => 501,
            Self::RoutePlan(_)             => 502,
            Self::PoolExhausted(_)         => 503,
            Self::Connect(_)               => 502,
            Self::Tls(_)                   => 525,
            Self::Http(_)                  => 502,
            Self::TooManyRedirects { .. }  => 310,
            Self::UpstreamStatus(s)        => *s,
            Self::WsUpgrade(_)             => 502,
            Self::WsFrame(_)               => 502,
            Self::Plugin { .. }            => 500,
            Self::Cache(_)                 => 500,
            Self::Config(_)                => 500,
            Self::Io(_)                    => 500,
            Self::Timeout(_)               => 504,
            Self::RateLimit(_)             => 429,
            Self::PayloadTooLarge { .. }   => 413,
            _                              => 500,
        }
    }
}
