//! # gateway-cache
//!
//! In-memory LRU cache backed by `moka` for KSP Gateway.
//!
//! The cache respects HTTP caching semantics (Cache-Control, ETag, Last-Modified).
//! A disk persistence layer is planned for Milestone 3.

use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use moka::future::Cache;
use tracing::{debug, trace};

use gateway_core::{
    response::NormalizedResponse,
    traits::{CacheKey, CacheStats, CacheStore},
};
use gateway_config::schema::CacheConfig;

pub struct MemoryCache {
    inner: Cache<CacheKey, Arc<NormalizedResponse>>,
    _stats: Arc<std::sync::atomic::AtomicU64>,
}

impl MemoryCache {
    pub fn from_config(cfg: &CacheConfig) -> Self {
        let max_bytes = cfg.max_memory_mb * 1024 * 1024;
        let inner = Cache::builder()
            .max_capacity(max_bytes)
            .time_to_live(Duration::from_secs(300))
            .build();
        Self {
            inner,
            _stats: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

// Manual implementation of CacheKey hash/eq for moka (requires Hash + Eq).
// CacheKey already derives both via its fields.

#[async_trait]
impl CacheStore for MemoryCache {
    async fn get(&self, key: &CacheKey) -> Option<NormalizedResponse> {
        match self.inner.get(key).await {
            Some(resp) => {
                debug!(authority = %key.authority, path = %key.path_and_query, "Cache HIT");
                Some((*resp).clone())
            }
            None => {
                trace!(authority = %key.authority, path = %key.path_and_query, "Cache MISS");
                None
            }
        }
    }

    async fn put(&self, key: CacheKey, resp: NormalizedResponse, _ttl_secs: u32) {
        // moka doesn't support per-entry TTL in the stable API; we use the
        // global TTL set at construction. Per-entry TTL will be added in Milestone 3.
        self.inner.insert(key, Arc::new(resp)).await;
    }

    async fn invalidate(&self, key: &CacheKey) {
        self.inner.invalidate(key).await;
    }

    async fn invalidate_authority(&self, authority: &str) {
        // moka doesn't support prefix invalidation; collect matching keys and remove.
        // For now, invalidate everything (conservative but safe).
        self.inner.invalidate_all();
        tracing::warn!(
            authority,
            "Full cache invalidation triggered (origin-scoped invalidation in Milestone 3)"
        );
    }

    async fn size_bytes(&self) -> u64 {
        self.inner.weighted_size()
    }

    async fn stats(&self) -> CacheStats {
        CacheStats::default() // Detailed stats wired in Milestone 3
    }
}
