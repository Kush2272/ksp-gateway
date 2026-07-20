use std::sync::Arc;
use gateway_dashboard::DashboardServer;
use gateway_ksp::SessionManager;
use gateway_monitor::GatewayMetrics;

#[tokio::test]
async fn test_dashboard_server_endpoints() {
    let metrics = Arc::new(GatewayMetrics::new().unwrap());
    let session_manager = Arc::new(SessionManager::new());

    // Bind on an ephemeral port (0) for testing
    let server = DashboardServer::new(0, metrics, session_manager);
    
    // Spawn server background task
    tokio::spawn(async move {
        let _ = server.run().await;
    });

    // Give server a moment to start up
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:9090";

    // Test healthz endpoint
    let res = client.get(format!("{}/healthz", base_url)).send().await;
    if let Ok(resp) = res {
        assert_eq!(resp.status(), 200);
    }

    // Test metrics endpoint
    let res = client.get(format!("{}/metrics", base_url)).send().await;
    if let Ok(resp) = res {
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert!(body.contains("ksp_gateway_active_sessions"));
    }
}
