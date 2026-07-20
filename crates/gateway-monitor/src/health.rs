//! Health check status.

use serde::Serialize;
use chrono::{DateTime, Utc};

/// Overall gateway health status, serialised to JSON for the `/healthz` endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status:       &'static str,    // "ok" | "degraded" | "down"
    pub version:      &'static str,
    pub uptime_secs:  u64,
    pub active_sessions: usize,
    pub checked_at:   DateTime<Utc>,
}

impl HealthStatus {
    pub fn ok(uptime_secs: u64, active_sessions: usize) -> Self {
        Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            uptime_secs,
            active_sessions,
            checked_at: Utc::now(),
        }
    }
}
