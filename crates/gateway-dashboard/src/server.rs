//! Axum server setup for the dashboard.

use std::{net::SocketAddr, sync::Arc};
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing::info;
use gateway_core::error::GatewayResult;
use gateway_monitor::GatewayMetrics;
use gateway_ksp::SessionManager;
use crate::routes;

pub struct DashboardServer {
    port:            u16,
    metrics:         Arc<GatewayMetrics>,
    session_manager: Arc<SessionManager>,
}

impl DashboardServer {
    pub fn new(
        port: u16,
        metrics: Arc<GatewayMetrics>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self { port, metrics, session_manager }
    }

    pub async fn run(self) -> GatewayResult<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        let metrics         = Arc::clone(&self.metrics);
        let session_manager = Arc::clone(&self.session_manager);

        let app = Router::new()
            .route("/",         get(routes::dashboard_index))
            .route("/healthz",  get(routes::healthz))
            .route("/metrics",  get(routes::prometheus_metrics))
            .route("/api/sessions", get(routes::api_sessions))
            .route("/api/stats",    get(routes::api_stats))
            .with_state(Arc::new(routes::DashboardState {
                metrics,
                session_manager,
                started_at: std::time::Instant::now(),
            }));

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            gateway_core::error::GatewayError::Internal(
                format!("Dashboard bind failed on {addr}: {e}")
            )
        })?;

        info!(addr = %addr, "Dashboard server listening");
        axum::serve(listener, app).await.map_err(|e| {
            gateway_core::error::GatewayError::Internal(
                format!("Dashboard server error: {e}")
            )
        })
    }
}
