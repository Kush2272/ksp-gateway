#[cfg(test)]
mod tests {
    use crate::GatewayResolver;
    use gateway_core::traits::Resolver;
    use gateway_config::schema::{ResolverConfig, HostOverride};

    #[tokio::test]
    async fn test_resolver_custom_hosts() {
        let cfg = ResolverConfig {
            use_doh: false,
            doh_url: "".to_string(),
            custom_hosts: vec![
                HostOverride {
                    host: "example.ksp".to_string(),
                    ip: "127.0.0.1".to_string(),
                }
            ],
        };

        let resolver = GatewayResolver::from_config(&cfg).await.unwrap();
        
        let ips = resolver.resolve("example.ksp").await.unwrap();
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0].to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_resolver_invalid_custom_host() {
        let cfg = ResolverConfig {
            use_doh: false,
            doh_url: "".to_string(),
            custom_hosts: vec![
                HostOverride {
                    host: "invalid.ksp".to_string(),
                    ip: "not-an-ip".to_string(),
                }
            ],
        };

        let result = GatewayResolver::from_config(&cfg).await;
        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid IP 'not-an-ip'"));
        } else {
            panic!("Expected error");
        }
    }
}
