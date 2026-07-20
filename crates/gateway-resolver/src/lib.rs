//! # gateway-resolver
//!
//! DNS resolution for KSP Gateway. Supports:
//! - System resolver (default)
//! - DNS-over-HTTPS via hickory-resolver
//! - Custom host overrides (applied before any DNS lookup)

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use async_trait::async_trait;
use dashmap::DashMap;
use tracing::debug;

use gateway_core::{error::{GatewayError, GatewayResult}, traits::Resolver};
use gateway_config::schema::{HostOverride, ResolverConfig};

// ─── Cache entry ──────────────────────────────────────────────────────────────

struct CacheEntry {
    addrs:   Vec<IpAddr>,
    expires: Instant,
}

// ─── GatewayResolver ──────────────────────────────────────────────────────────

/// The primary resolver implementation used by KSP Gateway.
///
/// Lookup order:
/// 1. Custom host overrides (from config)
/// 2. In-process TTL cache
/// 3. System DNS (or DoH if configured)
pub struct GatewayResolver {
    /// Custom host → IP overrides loaded from config.
    custom_hosts: HashMap<String, Vec<IpAddr>>,
    /// Simple TTL cache keyed by hostname.
    cache: DashMap<String, CacheEntry>,
    /// Positive cache TTL.
    cache_ttl: Duration,
    /// Underlying resolver (system or DoH).
    inner: Arc<hickory_resolver::TokioAsyncResolver>,
}

impl GatewayResolver {
    /// Build a `GatewayResolver` from the resolver configuration section.
    pub async fn from_config(cfg: &ResolverConfig) -> GatewayResult<Self> {
        let custom_hosts = parse_custom_hosts(&cfg.custom_hosts)?;

        let resolver_config = hickory_resolver::config::ResolverConfig::default();
        let mut opts = hickory_resolver::config::ResolverOpts::default();
        opts.cache_size = 1024;
        opts.timeout = Duration::from_secs(5);

        let inner = hickory_resolver::TokioAsyncResolver::tokio(resolver_config, opts);

        Ok(Self {
            custom_hosts,
            cache: DashMap::new(),
            cache_ttl: Duration::from_secs(300),
            inner: Arc::new(inner),
        })
    }
}

#[async_trait]
impl Resolver for GatewayResolver {
    async fn resolve(&self, host: &str) -> GatewayResult<Vec<IpAddr>> {
        // 1. Custom host overrides — highest priority.
        if let Some(addrs) = self.custom_hosts.get(host) {
            debug!(host, source = "custom_hosts", "Resolved");
            return Ok(addrs.clone());
        }

        // 2. In-process cache.
        if let Some(entry) = self.cache.get(host) {
            if entry.expires > Instant::now() {
                debug!(host, source = "cache", "Resolved");
                return Ok(entry.addrs.clone());
            }
        }

        // 3. DNS lookup.
        let lookup = self.inner.lookup_ip(host).await.map_err(|e| {
            GatewayError::DnsResolution {
                host:   host.to_owned(),
                reason: e.to_string(),
            }
        })?;

        let addrs: Vec<IpAddr> = lookup.iter().collect();
        if addrs.is_empty() {
            return Err(GatewayError::DnsResolution {
                host:   host.to_owned(),
                reason: "No addresses returned".into(),
            });
        }

        debug!(host, addrs = ?addrs, source = "dns", "Resolved");

        // Store in cache.
        self.cache.insert(host.to_owned(), CacheEntry {
            addrs:   addrs.clone(),
            expires: Instant::now() + self.cache_ttl,
        });

        Ok(addrs)
    }
}

fn parse_custom_hosts(overrides: &[HostOverride]) -> GatewayResult<HashMap<String, Vec<IpAddr>>> {
    let mut map = HashMap::new();
    for entry in overrides {
        let ip: IpAddr = entry.ip.parse().map_err(|e| {
            GatewayError::Config(format!(
                "Invalid IP '{}' for custom host '{}': {e}",
                entry.ip, entry.host
            ))
        })?;
        map.entry(entry.host.clone()).or_insert_with(Vec::new).push(ip);
    }
    Ok(map)
}

#[cfg(test)]
mod tests;
