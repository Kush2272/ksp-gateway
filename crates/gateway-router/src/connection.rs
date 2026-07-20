//! Connection manager — acquires connections from the pool.

use std::sync::Arc;
use gateway_core::{
    error::GatewayResult,
    traits::RouteDecision,
};
use crate::pool::{ConnectionPool, OriginClient};

pub struct GatewayConnectionManager {
    pool: Arc<ConnectionPool>,
}

impl GatewayConnectionManager {
    pub fn new() -> Self {
        Self { pool: Arc::new(ConnectionPool::new()) }
    }

    pub async fn acquire(&self, decision: &RouteDecision) -> GatewayResult<Arc<OriginClient>> {
        self.pool.acquire(&decision.target)
    }

    pub fn pool_size(&self) -> usize {
        self.pool.size()
    }
}

impl Default for GatewayConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
