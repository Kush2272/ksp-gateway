//! Route handlers for the dashboard Axum server.

use std::{sync::Arc, time::Instant};
use axum::{extract::State, response::{Html, IntoResponse, Json}, http::StatusCode};
use serde_json::json;
use gateway_monitor::{GatewayMetrics, HealthStatus};
use gateway_ksp::SessionManager;
use crate::templates;

pub struct DashboardState {
    pub metrics:         Arc<GatewayMetrics>,
    pub session_manager: Arc<SessionManager>,
    pub started_at:      Instant,
}

pub async fn dashboard_index(
    State(state): State<Arc<DashboardState>>,
) -> Html<String> {
    Html(templates::dashboard_html(
        state.session_manager.active_count(),
        state.started_at.elapsed().as_secs(),
    ))
}

pub async fn healthz(
    State(state): State<Arc<DashboardState>>,
) -> impl IntoResponse {
    let health = HealthStatus::ok(
        state.started_at.elapsed().as_secs(),
        state.session_manager.active_count(),
    );
    Json(health)
}

pub async fn prometheus_metrics(
    State(state): State<Arc<DashboardState>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        state.metrics.render(),
    )
}

pub async fn api_sessions(
    State(state): State<Arc<DashboardState>>,
) -> impl IntoResponse {
    let sessions: Vec<_> = state
        .session_manager
        .sessions()
        .iter()
        .map(|s| json!({
            "id":           s.id.to_string(),
            "peer":         s.peer_addr.to_string(),
            "connected_at": s.connected_at.to_rfc3339(),
            "requests":     s.requests,
            "bytes_in":     s.bytes_in,
            "bytes_out":    s.bytes_out,
        }))
        .collect();
    Json(sessions)
}

pub async fn api_stats(
    State(state): State<Arc<DashboardState>>,
) -> impl IntoResponse {
    Json(json!({
        "active_sessions": state.session_manager.active_count(),
        "uptime_secs":     state.started_at.elapsed().as_secs(),
        "version":         env!("CARGO_PKG_VERSION"),
    }))
}
