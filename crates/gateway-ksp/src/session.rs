//! Session manager — tracks active KSP sessions and handles request dispatch.

use std::{net::SocketAddr, sync::Arc};
use dashmap::DashMap;
use tokio::net::TcpStream;
use tracing::{debug, info};
use gateway_core::{
    error::GatewayResult,
    types::SessionId,
};

/// Metadata about a single active KSP session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id:        SessionId,
    pub peer_addr: SocketAddr,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub requests:  u64,
    pub bytes_in:  u64,
    pub bytes_out: u64,
}

/// Manages all active KSP sessions.
pub struct SessionManager {
    sessions: DashMap<SessionId, SessionInfo>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: DashMap::new() }
    }

    /// Called by `KspListener` for each accepted TCP connection.
    ///
    /// Performs the KSP handshake (Milestone 2), then enters the
    /// request dispatch loop.
    pub async fn handle_connection(
        self: Arc<Self>,
        _stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> GatewayResult<()> {
        let session_id = SessionId::new();

        let info = SessionInfo {
            id: session_id,
            peer_addr,
            connected_at: chrono::Utc::now(),
            requests:  0,
            bytes_in:  0,
            bytes_out: 0,
        };

        self.sessions.insert(session_id, info);
        info!(session = %session_id, peer = %peer_addr, "Session established");

        // Milestone 2: integrate ksp-server handshake and request loop here.
        // For now, drop the connection after logging.
        debug!(session = %session_id, "Session handler placeholder — Milestone 2");

        self.sessions.remove(&session_id);
        info!(session = %session_id, "Session closed");
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn sessions(&self) -> Vec<SessionInfo> {
        self.sessions.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
