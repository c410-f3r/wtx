//! WebSocket CLI client.

#[path = "./common/mod.rs"]
mod common;
#[path = "./tls_stream/mod.rs"]
mod tls_stream;

use tokio::io::{AsyncBufReadExt, BufReader};
use wtx::{
  misc::UriPartsRef,
  rng::StdRng,
  web_socket::{
    handshake::{WebSocketConnect, WebSocketConnectRaw},
    FrameBufferVec, FrameMutVec, OpCode, WebSocketBuffer,
  },
};

#[tokio::main]
async fn main() {
  let fb = &mut FrameBufferVec::default();
  let uri = common::_uri_from_args();
  let uri_parts = UriPartsRef::new(uri.as_str());
  let (_, mut ws) = WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut <_>::default(),
    rng: StdRng::default(),
    stream: tls_stream::_tls_stream_host(uri_parts.host(), uri_parts.hostname()).await,
    uri: &uri,
    wsb: WebSocketBuffer::default(),
  }
  .connect()
  .await
  .unwrap();
  let mut buffer = String::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame(fb) => {
        let frame = frame_rslt.unwrap();
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => println!("{elem}"),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = reader.read_line(&mut buffer) => {
        let _ = read_rslt.unwrap();
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, buffer.as_bytes()).unwrap()).await.unwrap();
      }
    }
  }
}
