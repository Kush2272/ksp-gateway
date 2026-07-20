//! KSP frame codec — translates between KSP wire frames and NormalizedRequest/Response.
//! Full implementation in Milestone 2.

use bytes::Bytes;
use gateway_core::{
    error::GatewayResult,
    request::{HttpMethod, NormalizedRequest},
    response::NormalizedResponse,
    types::Authority,
};

/// Decode a KSP DATA payload into a `NormalizedRequest`.
///
/// The payload format (Milestone 2): length-prefixed header map followed by body.
pub fn decode_request(_payload: Bytes) -> GatewayResult<NormalizedRequest> {
    // Placeholder: Milestone 2 will parse the actual KSP DATA frame.
    Ok(NormalizedRequest {
        method:         HttpMethod::Get,
        path_and_query: "/".into(),
        http_version:   1,
        headers:        std::collections::HashMap::new(),
        body:           None,
        authority:      Authority::https("placeholder.invalid"),
    })
}

/// Encode a `NormalizedResponse` into a KSP DATA payload.
///
/// For streaming responses this will be called once per chunk.
pub fn encode_response(resp: &NormalizedResponse) -> GatewayResult<Bytes> {
    // Placeholder: Milestone 2 will serialize into the KSP frame format.
    let line = format!(
        "HTTP/1.1 {} {}\r\n\r\n",
        resp.status, resp.reason
    );
    Ok(Bytes::from(line))
}
