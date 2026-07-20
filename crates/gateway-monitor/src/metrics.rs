//! Prometheus metrics registry for KSP Gateway.

use prometheus::{
    CounterVec, GaugeVec, HistogramVec, IntGauge, Registry,
    Opts, HistogramOpts,
};

/// All Prometheus metrics for the gateway, registered in a single `Registry`.
pub struct GatewayMetrics {
    pub registry:          Registry,
    pub active_sessions:   IntGauge,
    pub requests_total:    CounterVec,
    pub request_duration:  HistogramVec,
    pub response_status:   CounterVec,
    pub cache_hits:        CounterVec,
    pub pool_connections:  GaugeVec,
}

impl GatewayMetrics {
    pub fn new() -> prometheus::Result<Self> {
        let registry = Registry::new();

        let active_sessions = IntGauge::new(
            "ksp_gateway_active_sessions",
            "Number of currently active KSP sessions",
        )?;

        let requests_total = CounterVec::new(
            Opts::new("ksp_gateway_requests_total", "Total number of proxied requests"),
            &["method", "pipeline"],
        )?;

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "ksp_gateway_request_duration_ms",
                "End-to-end request latency in milliseconds",
            )
            .buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 5000.0]),
            &["pipeline"],
        )?;

        let response_status = CounterVec::new(
            Opts::new("ksp_gateway_response_status_total", "Response status codes"),
            &["status_class"],
        )?;

        let cache_hits = CounterVec::new(
            Opts::new("ksp_gateway_cache_total", "Cache hits and misses"),
            &["result"], // "hit" | "miss"
        )?;

        let pool_connections = GaugeVec::new(
            Opts::new("ksp_gateway_pool_connections", "Active upstream connections per origin"),
            &["origin"],
        )?;

        // Register all metrics.
        registry.register(Box::new(active_sessions.clone()))?;
        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(response_status.clone()))?;
        registry.register(Box::new(cache_hits.clone()))?;
        registry.register(Box::new(pool_connections.clone()))?;

        Ok(Self {
            registry,
            active_sessions,
            requests_total,
            request_duration,
            response_status,
            cache_hits,
            pool_connections,
        })
    }

    /// Render all metrics in the Prometheus text exposition format.
    pub fn render(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buf = Vec::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buf).unwrap_or_default();
        String::from_utf8(buf).unwrap_or_default()
    }
}

impl Default for GatewayMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to register Prometheus metrics")
    }
}
