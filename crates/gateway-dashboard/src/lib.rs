//! # gateway-dashboard
//!
//! Axum HTTP server serving the KSP Gateway web dashboard at `localhost:9090`.
//!
//! All pages are rendered as single-file HTML/CSS/JS embedded in the binary —
//! no external asset server or framework is required.

pub mod server;
pub mod routes;
pub mod templates;

pub use server::DashboardServer;
