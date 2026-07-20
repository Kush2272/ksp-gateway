//! # gateway-ksp
//!
//! KSP session listener, frame codec, and session manager for KSP Gateway.
//!
//! This crate is the entry point for all incoming browser connections.
//! It wraps `ksp-server` to accept sessions, decodes request frames,
//! routes them to the correct pipeline (HTTP or WS), and encodes
//! the response back into KSP data frames.

pub mod listener;
pub mod session;
pub mod codec;
pub mod router;

pub use listener::KspListener;
pub use session::SessionManager;
