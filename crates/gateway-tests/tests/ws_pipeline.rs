use bytes::Bytes;
use gateway_pipeline_ws::frame::{WsFrame, WsFrameKind};
use tokio_tungstenite::tungstenite::Message;

#[test]
fn test_ws_frame_conversion() {
    let text_frame = WsFrame::text("hello websocket");
    assert_eq!(text_frame.kind, WsFrameKind::Text);
    
    let msg = text_frame.to_message();
    match &msg {
        Message::Text(s) => assert_eq!(s, "hello websocket"),
        _ => panic!("Expected text message"),
    }

    let converted_back = WsFrame::from_message(msg).unwrap();
    assert_eq!(converted_back.payload, Bytes::from("hello websocket"));
}

#[test]
fn test_ws_binary_frame_conversion() {
    let bin_frame = WsFrame::binary(vec![1, 2, 3, 4]);
    assert_eq!(bin_frame.kind, WsFrameKind::Binary);

    let msg = bin_frame.to_message();
    match &msg {
        Message::Binary(b) => assert_eq!(b, &[1, 2, 3, 4]),
        _ => panic!("Expected binary message"),
    }
}
