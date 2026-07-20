//! Per-origin connection pool backed by reqwest client pooling.

use std::sync::Arc;
use dashmap::DashMap;
use reqwest::Client;
use gateway_core::{error::{GatewayError, GatewayResult}, types::Authority};

/// A client wrapper holding a pooled reqwest HTTP client.
pub struct OriginClient {
    pub authority: Authority,
    pub client:    Client,
}

impl OriginClient {
    pub fn new(authority: Authority) -> GatewayResult<Self> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|e| GatewayError::Internal(format!("Failed to create reqwest client: {e}")))?;

        Ok(Self { authority, client })
    }
}

/// Per-origin pool of `OriginClient` instances.
pub struct ConnectionPool {
    pool: DashMap<String, Arc<OriginClient>>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self { pool: DashMap::new() }
    }

    /// Acquire a connection wrapper for `authority`, creating one if needed.
    pub fn acquire(&self, authority: &Authority) -> GatewayResult<Arc<OriginClient>> {
        let key = authority.to_string();
        if let Some(client) = self.pool.get(&key) {
            return Ok(Arc::clone(client.value()));
        }
        let client = Arc::new(OriginClient::new(authority.clone())?);
        self.pool.insert(key, Arc::clone(&client));
        Ok(client)
    }

    pub fn size(&self) -> usize {
        self.pool.len()
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}
