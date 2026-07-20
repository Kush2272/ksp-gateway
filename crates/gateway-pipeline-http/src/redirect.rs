//! Redirect follower — handles 3xx responses up to the configured hop limit.

use gateway_core::{error::{GatewayError, GatewayResult}, response::NormalizedResponse};

pub struct RedirectFollower {
    max_hops: u8,
}

impl RedirectFollower {
    pub fn new(max_hops: u8) -> Self {
        Self { max_hops }
    }

    /// Given a redirect response, extract the new URL and validate the hop count.
    pub fn next_url(
        &self,
        resp: &NormalizedResponse,
        hop: u8,
    ) -> GatewayResult<Option<String>> {
        if !resp.is_redirect() {
            return Ok(None);
        }
        if hop >= self.max_hops {
            return Err(GatewayError::TooManyRedirects {
                max:  self.max_hops,
                seen: hop,
            });
        }
        Ok(resp.location().map(String::from))
    }
}
