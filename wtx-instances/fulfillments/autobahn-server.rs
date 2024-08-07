//! WebSocket autobahn server.

use tokio::net::TcpStream;
use wtx::{
  http::LowLevelServer,
  rng::StdRng,
  web_socket::{
    compression::{Flate2, NegotiatedFlate2},
    FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServer,
  },
};

#[tokio::main]
async fn main() {
  LowLevelServer::tokio_web_socket(
    "127.0.0.1:9070",
    None,
    Flate2::default,
    |err| eprintln!("Connection error: {err:?}"),
    handle,
    (|| {}, |_| {}, |_, stream| async move { Ok(stream) }),
  )
  .await
  .unwrap()
}

async fn handle(
  (fb, mut ws): (
    &mut FrameBufferVec,
    WebSocketServer<Option<NegotiatedFlate2>, StdRng, TcpStream, &mut WebSocketBuffer>,
  ),
) -> wtx::Result<()> {
  loop {
    let mut frame = ws.read_frame(fb).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        ws.write_frame(&mut frame).await?;
      }
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}
