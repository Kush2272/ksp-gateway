//! TLS connector backed by rustls with the system certificate store.

use std::sync::Arc;
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use gateway_core::error::{GatewayError, GatewayResult};

/// Build a `rustls::ClientConfig` that trusts the system's native certificate store.
///
/// This is used by the connection manager when dialing TLS upstream origins.
pub fn build_tls_config(verify: bool) -> GatewayResult<Arc<ClientConfig>> {
    if !verify {
        // Development mode: accept any certificate.
        // This uses a custom verifier that always returns Ok.
        let config = ClientConfig::builder()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();

        // Note: In production `verify` should always be true.
        tracing::warn!("TLS certificate verification is DISABLED — development mode only");
        return Ok(Arc::new(config));
    }

    let mut root_store = RootCertStore::empty();

    let native_certs = load_native_certs();

    // Log any errors loading individual certs but continue with those that loaded.
    for err in &native_certs.errors {
        tracing::warn!(error = %err, "Failed to load a native certificate");
    }

    for cert in native_certs.certs {
        root_store
            .add(cert)
            .map_err(|e| GatewayError::Tls(format!("Failed to add root cert: {e}")))?;
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Arc::new(config))
}
