//! Configuration schema — the full `GatewayConfig` struct tree.
//!
//! Every field has a serde `default` annotation so that a minimal TOML file
//! is valid. The `GatewayConfig::validate()` method catches logical errors
//! (e.g., pool_size = 0) at startup rather than at runtime.

use serde::{Deserialize, Serialize};

// ─── Top-level ─────────────────────────────────────────────────────────────────

/// Root configuration struct for KSP Gateway.
///
/// Loaded from TOML (typically `config/default.toml`) and optionally
/// overridden by environment variables with the prefix `KSP_GW_`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub ksp:       KspConfig,
    pub router:    RouterConfig,
    pub resolver:  ResolverConfig,
    pub pipeline:  PipelineConfig,
    pub cache:     CacheConfig,
    pub plugins:   PluginsConfig,
    pub monitor:   MonitorConfig,
    pub tls:       TlsConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            ksp:      KspConfig::default(),
            router:   RouterConfig::default(),
            resolver: ResolverConfig::default(),
            pipeline: PipelineConfig::default(),
            cache:    CacheConfig::default(),
            plugins:  PluginsConfig::default(),
            monitor:  MonitorConfig::default(),
            tls:      TlsConfig::default(),
        }
    }
}

impl GatewayConfig {
    /// Validate configuration values for logical correctness.
    ///
    /// Returns `Err` with a human-readable description of the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if self.ksp.max_sessions == 0 {
            return Err("[ksp] max_sessions must be > 0".into());
        }
        if self.pipeline.http.pool_size == 0 {
            return Err("[pipeline.http] pool_size must be > 0".into());
        }
        if self.pipeline.http.request_timeout_secs == 0 {
            return Err("[pipeline.http] request_timeout_secs must be > 0".into());
        }
        if self.monitor.prometheus_port == 0 {
            return Err("[monitor] prometheus_port must be > 0".into());
        }
        Ok(())
    }
}

// ─── KSP listener ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KspConfig {
    /// Address and port to listen on for incoming KSP connections.
    pub listen: String,
    /// Maximum number of concurrent KSP sessions.
    pub max_sessions: u32,
    /// Idle session timeout in seconds. Sessions inactive longer than this are closed.
    pub session_ttl_secs: u32,
}

impl Default for KspConfig {
    fn default() -> Self {
        Self {
            listen:          "0.0.0.0:8765".into(),
            max_sessions:    100_000,
            session_ttl_secs: 300,
        }
    }
}

// ─── Router ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RouterConfig {
    /// Maximum number of HTTP redirects to follow before returning an error.
    pub max_redirects: u8,
    /// Per-request timeout in seconds (from KSP receive to first upstream byte).
    pub request_timeout_secs: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_redirects:        10,
            request_timeout_secs: 30,
        }
    }
}

// ─── Resolver ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolverConfig {
    /// Use DNS-over-HTTPS instead of the system resolver.
    pub use_doh: bool,
    /// DoH endpoint URL.
    pub doh_url: String,
    /// Custom host overrides. Applied before DNS resolution.
    pub custom_hosts: Vec<HostOverride>,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            use_doh:      false,
            doh_url:      "https://cloudflare-dns.com/dns-query".into(),
            custom_hosts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOverride {
    pub host: String,
    pub ip:   String,
}

// ─── Pipeline ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineConfig {
    pub http: HttpPipelineConfig,
    pub ws:   WsPipelineConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            http: HttpPipelineConfig::default(),
            ws:   WsPipelineConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpPipelineConfig {
    pub enabled:              bool,
    /// Maximum connections per origin in the connection pool.
    pub pool_size:            usize,
    /// Per-request timeout in seconds.
    pub request_timeout_secs: u64,
    /// User-Agent header to send to origins.
    pub user_agent:           String,
    /// Prefer HTTP/2 over HTTP/1.1 when the origin supports it.
    pub prefer_http2:         bool,
}

impl Default for HttpPipelineConfig {
    fn default() -> Self {
        Self {
            enabled:              true,
            pool_size:            256,
            request_timeout_secs: 30,
            user_agent:           "KSP-Gateway/0.1.0".into(),
            prefer_http2:         true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WsPipelineConfig {
    pub enabled: bool,
    /// Maximum WebSocket message size in bytes.
    pub max_message_size: usize,
}

impl Default for WsPipelineConfig {
    fn default() -> Self {
        Self {
            enabled:          true,
            max_message_size: 64 * 1024 * 1024, // 64 MB
        }
    }
}

// ─── Cache ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled:       bool,
    pub max_memory_mb: u64,
    pub disk_path:     Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled:       true,
            max_memory_mb: 512,
            disk_path:     None,
        }
    }
}

// ─── Plugins ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    /// Ordered list of plugin names to enable. Execution order is determined
    /// by each plugin's declared priority, not this list order.
    pub enabled: Vec<String>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "logging".into(),
                "metrics".into(),
                "headers".into(),
                "compression".into(),
                "cache".into(),
                "rate-limit".into(),
            ],
        }
    }
}

// ─── Monitor ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    pub prometheus_port: u16,
    /// Serve the web dashboard alongside Prometheus metrics.
    pub dashboard:       bool,
    /// Log format: "json" or "pretty".
    pub log_format:      String,
    /// Minimum log level: "trace", "debug", "info", "warn", "error".
    pub log_level:       String,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            prometheus_port: 9090,
            dashboard:       true,
            log_format:      "pretty".into(),
            log_level:       "info".into(),
        }
    }
}

// ─── TLS ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TlsConfig {
    /// Verify upstream TLS certificates. Should only be `false` in dev.
    pub verify_upstream: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self { verify_upstream: true }
    }
}
