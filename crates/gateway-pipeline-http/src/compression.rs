//! Compression negotiation — gzip, brotli, zstd.
//! Full streaming implementation in Milestone 3.

/// Encoding types supported by the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Gzip,
    Brotli,
    Zstd,
    Identity,
}

impl Encoding {
    pub fn from_content_encoding(header: &str) -> Self {
        match header.trim() {
            "gzip"     => Self::Gzip,
            "br"       => Self::Brotli,
            "zstd"     => Self::Zstd,
            _           => Self::Identity,
        }
    }

    pub fn accept_encoding_header() -> &'static str {
        "gzip, br, zstd"
    }
}
