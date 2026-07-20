//! Protocol adapter — translates NormalizedRequest into real HTTP requests via OriginClient.

use std::{collections::HashMap, str::FromStr, sync::Arc};
use reqwest::{header::{HeaderName, HeaderValue}, Method, RequestBuilder};

use gateway_core::{
    error::{GatewayError, GatewayResult},
    request::{HttpMethod, NormalizedRequest},
    response::NormalizedResponse,
};
use crate::pool::OriginClient;

/// Adapter that executes HTTP requests against upstream origins.
pub struct HttpAdapter {
    client: Arc<OriginClient>,
}

impl HttpAdapter {
    pub fn new(client: Arc<OriginClient>) -> Self {
        Self { client }
    }

    pub async fn send(&mut self, req: NormalizedRequest) -> GatewayResult<NormalizedResponse> {
        let scheme = if req.authority.tls { "https" } else { "http" };
        let url = format!("{}://{}{}", scheme, req.authority.to_string(), req.path_and_query);

        let method = match req.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Trace => Method::TRACE,
            HttpMethod::Connect => Method::CONNECT,
            HttpMethod::Other(ref s) => Method::from_bytes(s.as_bytes())
                .map_err(|e| GatewayError::Internal(format!("Invalid HTTP method '{s}': {e}")))?,
        };

        let mut req_builder: RequestBuilder = self.client.client.request(method, &url);

        for (k, v) in &req.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                req_builder = req_builder.header(name, val);
            }
        }

        if let Some(body) = req.body {
            req_builder = req_builder.body(body);
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| GatewayError::Http(format!("Upstream request to {url} failed: {e}")))?;

        let status = response.status().as_u16();
        let reason = response.status().canonical_reason().unwrap_or("OK").to_string();

        let mut headers = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.insert(k.as_str().to_lowercase(), v_str.to_string());
            }
        }

        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| GatewayError::Http(format!("Failed to read response body: {e}")))?;

        Ok(NormalizedResponse {
            status,
            reason,
            http_version: 1, // HTTP/1.1 or HTTP/2
            headers,
            body: Some(body_bytes),
            is_streaming: false,
        })
    }
}
