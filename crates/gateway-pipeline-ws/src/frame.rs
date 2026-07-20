//! WebSocket frame type definitions.

/// A WebSocket frame type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsFrameKind {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

/// A decoded WebSocket frame ready for relay.
#[derive(Debug, Clone)]
pub struct WsFrame {
    pub kind:    WsFrameKind,
    pub payload: bytes::Bytes,
}
