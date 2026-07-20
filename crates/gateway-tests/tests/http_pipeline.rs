use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use gateway_config::schema::{ResolverConfig, RouterConfig};
use gateway_resolver::GatewayResolver;
use gateway_core::{
    request::{HttpMethod, NormalizedRequest, RequestContext},
    types::{Authority, PipelineKind, SessionId, StreamId},
};
use gateway_plugin::PluginChain;
use gateway_pipeline_http::HttpPipeline;
use gateway_router::{DefaultRoutePlanner, GatewayConnectionManager};

async fn spawn_mock_http_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n > 0 {
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello Gateway";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    addr
}

#[tokio::test]
async fn test_http_pipeline_end_to_end() {
    let server_addr = spawn_mock_http_server().await;

    let resolver_cfg = ResolverConfig {
        use_doh: false,
        doh_url: "".to_string(),
        custom_hosts: vec![],
    };
    let resolver = Arc::new(GatewayResolver::from_config(&resolver_cfg).await.unwrap());
    let router_cfg = Arc::new(RouterConfig {
        max_redirects: 5,
        request_timeout_secs: 30,
    });

    let plugins = Arc::new(PluginChain::new(vec![]));
    let planner = Arc::new(DefaultRoutePlanner::new(resolver, router_cfg));
    let conn_manager = Arc::new(GatewayConnectionManager::new());

    let pipeline = HttpPipeline::new(plugins, planner, conn_manager);

    let ctx = RequestContext::new(
        SessionId::new(),
        StreamId(1),
        PipelineKind::Http,
        server_addr,
    );

    let req = NormalizedRequest {
        method: HttpMethod::Get,
        path_and_query: "/".to_string(),
        http_version: 1,
        headers: HashMap::new(),
        body: None,
        authority: Authority {
            host: server_addr.ip().to_string(),
            port: server_addr.port(),
            tls: false,
        },
    };

    let resp = pipeline.handle(ctx, req).await.expect("Pipeline handle failed");

    assert_eq!(resp.status, 200);
    assert_eq!(resp.headers.get("content-type").unwrap(), "text/plain");
    assert_eq!(resp.body.as_ref().unwrap().as_ref(), b"Hello Gateway");
}
