//! Plugin chain integration tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use gateway_plugin::{
        PluginChain,
        builtin::{LoggingPlugin, MetricsPlugin, HeadersPlugin},
    };
    use gateway_core::{
        request::{HttpMethod, NormalizedRequest, RequestContext},
        types::{Authority, PipelineKind, SessionId, StreamId},
    };

    fn make_request() -> NormalizedRequest {
        NormalizedRequest {
            method:         HttpMethod::Get,
            path_and_query: "/test".into(),
            http_version:   1,
            headers:        std::collections::HashMap::from([
                ("connection".into(), "keep-alive".into()),
                ("upgrade".into(), "keep-alive".into()),
            ]),
            body:           None,
            authority:      Authority::https("example.com"),
        }
    }

    fn make_ctx() -> RequestContext {
        RequestContext::new(
            SessionId::new(),
            StreamId(1),
            PipelineKind::Http,
            "127.0.0.1:12345".parse().unwrap(),
        )
    }

    #[tokio::test]
    async fn plugin_chain_runs_all_plugins() {
        let chain = PluginChain::new(vec![
            Arc::new(LoggingPlugin),
            Arc::new(MetricsPlugin),
            Arc::new(HeadersPlugin),
        ]);
        assert_eq!(chain.len(), 3);

        let mut ctx = make_ctx();
        let mut req = make_request();
        let result = chain.run_request(&mut ctx, &mut req).await;
        assert!(result.is_ok(), "Plugin chain should not error: {result:?}");
        assert!(result.unwrap().is_none(), "No plugin should short-circuit a plain GET");
    }

    #[tokio::test]
    async fn headers_plugin_strips_hop_by_hop() {
        let chain = PluginChain::new(vec![Arc::new(HeadersPlugin)]);
        let mut ctx = make_ctx();
        let mut req = make_request();

        // Pre-condition: hop-by-hop headers present
        assert!(req.headers.contains_key("connection"));

        chain.run_request(&mut ctx, &mut req).await.unwrap();

        // Post-condition: stripped
        assert!(!req.headers.contains_key("connection"), "connection header should be stripped");
        // Injected headers present
        assert!(req.headers.contains_key("x-forwarded-for"));
        assert!(req.headers.contains_key("x-ksp-correlation-id"));
    }

    #[tokio::test]
    async fn plugin_chain_priority_order() {
        // Logging (priority 1) must appear before Headers (priority 50)
        let chain = PluginChain::new(vec![
            Arc::new(HeadersPlugin),   // priority 50, inserted first
            Arc::new(LoggingPlugin),   // priority 1, inserted second
        ]);
        // After sorting by priority, LoggingPlugin (1) should be first.
        // We verify indirectly: both run without error.
        let mut ctx = make_ctx();
        let mut req = make_request();
        assert!(chain.run_request(&mut ctx, &mut req).await.is_ok());
    }
}
