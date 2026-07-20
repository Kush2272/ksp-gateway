//! WebSocket frame type definitions and codecs.

use bytes::Bytes;
use tokio_tungstenite::tungstenite::Message;

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
    pub payload: Bytes,
}

impl WsFrame {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            kind: WsFrameKind::Text,
            payload: Bytes::from(s.into()),
        }
    }

    pub fn binary(b: impl Into<Bytes>) -> Self {
        Self {
            kind: WsFrameKind::Binary,
            payload: b.into(),
        }
    }

    pub fn to_message(&self) -> Message {
        match self.kind {
            WsFrameKind::Text => Message::Text(String::from_utf8_lossy(&self.payload).to_string()),
            WsFrameKind::Binary => Message::Binary(self.payload.to_vec()),
            WsFrameKind::Ping => Message::Ping(self.payload.to_vec()),
            WsFrameKind::Pong => Message::Pong(self.payload.to_vec()),
            WsFrameKind::Close => Message::Close(None),
        }
    }

    pub fn from_message(msg: Message) -> Option<Self> {
        match msg {
            Message::Text(s) => Some(Self {
                kind: WsFrameKind::Text,
                payload: Bytes::from(s),
            }),
            Message::Binary(b) => Some(Self {
                kind: WsFrameKind::Binary,
                payload: Bytes::from(b),
            }),
            Message::Ping(p) => Some(Self {
                kind: WsFrameKind::Ping,
                payload: Bytes::from(p),
            }),
            Message::Pong(p) => Some(Self {
                kind: WsFrameKind::Pong,
                payload: Bytes::from(p),
            }),
            Message::Close(_) => Some(Self {
                kind: WsFrameKind::Close,
                payload: Bytes::new(),
            }),
            Message::Frame(_) => None,
        }
    }
}
