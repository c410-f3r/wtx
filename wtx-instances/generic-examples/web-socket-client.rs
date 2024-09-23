//! WebSocket CLI client that enables real-time communication by allowing users to send and
//! receive messages through typing.
//!
//! This snippet requires ~35 dependencies and has an optimized binary size of ~550K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
};
use wtx::{
  misc::{simple_seed, Uri, Xorshift64},
  web_socket::{
    FrameBufferVec, FrameMutVec, HeadersBuffer, OpCode, WebSocketBuffer, WebSocketClient,
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("ws://www.example.com");
  let fb = &mut FrameBufferVec::default();
  let (_, mut ws) = WebSocketClient::connect(
    (),
    fb,
    [],
    &mut HeadersBuffer::default(),
    Xorshift64::from(simple_seed()),
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
    &uri.to_ref(),
    WebSocketBuffer::default(),
  )
  .await?;
  let mut buffer = String::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame(fb) => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => println!("{elem}"),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = reader.read_line(&mut buffer) => {
        let _ = read_rslt?;
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, buffer.as_bytes())?).await?;
      }
    }
  }
  Ok(())
}
