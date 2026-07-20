//! Per-origin connection pool using hyper's connection management.

use std::sync::Arc;
use dashmap::DashMap;

use gateway_core::types::Authority;

/// A simple wrapper holding a hyper HTTPS client that can be reused across requests.
///
/// In the full implementation this will be a proper pooled connection manager.
/// For Milestone 1 this provides a compilable skeleton.
pub struct OriginClient {
    pub authority: Authority,
}

impl OriginClient {
    pub fn new(authority: Authority) -> Self {
        Self { authority }
    }
}

/// Per-origin pool of `OriginClient` instances.
///
/// DashMap provides lock-free concurrent access to the pool entries.
pub struct ConnectionPool {
    pool: DashMap<String, Arc<OriginClient>>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self { pool: DashMap::new() }
    }

    /// Acquire a connection to `authority`, creating one if needed.
    pub fn acquire(&self, authority: &Authority) -> Arc<OriginClient> {
        let key = authority.to_string();
        if let Some(client) = self.pool.get(&key) {
            return Arc::clone(client.value());
        }
        let client = Arc::new(OriginClient::new(authority.clone()));
        self.pool.insert(key, Arc::clone(&client));
        client
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
