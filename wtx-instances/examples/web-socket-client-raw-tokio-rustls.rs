//! WebSocket CLI client.

use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
};
use wtx::{
  misc::{TokioRustlsConnector, UriString},
  rng::StdRng,
  web_socket::{
    FrameBufferVec, FrameMutVec, HeadersBuffer, OpCode, WebSocketBuffer, WebSocketClient,
  },
};

#[tokio::main]
async fn main() {
  let fb = &mut FrameBufferVec::default();
  let uri = UriString::new(wtx_instances::uri_from_args());
  let (_, mut ws) = WebSocketClient::connect(
    (),
    fb,
    [],
    &mut HeadersBuffer::default(),
    StdRng::default(),
    TokioRustlsConnector::from_auto()
      .unwrap()
      .push_certs(include_bytes!("../../.certs/root-ca.crt"))
      .unwrap()
      .connect_without_client_auth(uri.hostname(), TcpStream::connect(uri.host()).await.unwrap())
      .await
      .unwrap(),
    &uri.to_ref(),
    WebSocketBuffer::default(),
  )
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
