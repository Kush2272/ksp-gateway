//! KSP Listener — accepts incoming TCP connections and performs the KSP handshake.
//!
//! Full implementation in Milestone 2. This skeleton compiles and establishes
//! the type signatures used by the CLI and tests.

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{error, info};
use gateway_core::error::GatewayResult;
use crate::session::SessionManager;

pub struct KspListener {
    addr:            SocketAddr,
    session_manager: Arc<SessionManager>,
}

impl KspListener {
    pub fn new(addr: SocketAddr, session_manager: Arc<SessionManager>) -> Self {
        Self { addr, session_manager }
    }

    /// Bind to the configured address and begin accepting KSP connections.
    ///
    /// This future runs indefinitely until the process is shut down.
    pub async fn run(self) -> GatewayResult<()> {
        let listener = TcpListener::bind(self.addr).await.map_err(|e| {
            gateway_core::error::GatewayError::Internal(
                format!("Failed to bind KSP listener on {}: {e}", self.addr)
            )
        })?;

        info!(addr = %self.addr, "KSP Gateway listening");

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let manager = Arc::clone(&self.session_manager);
                    tokio::spawn(async move {
                        if let Err(e) = manager.handle_connection(stream, peer_addr).await {
                            error!(peer = %peer_addr, error = %e, "Session error");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "Accept error");
                }
            }
        }
    }
}
