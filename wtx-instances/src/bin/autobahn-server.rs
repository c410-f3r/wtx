//! WebSocket autobahn server.

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::TcpStream;
use wtx::{
  http::LowLevelServer,
  misc::Xorshift64,
  web_socket::{
    compression::{Flate2, NegotiatedFlate2},
    FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServer,
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  LowLevelServer::tokio_web_socket(
    "127.0.0.1:9070",
    None,
    Flate2::default,
    |error| eprintln!("{error}"),
    handle,
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream) }),
  )
  .await
}

async fn handle(
  fb: &mut FrameBufferVec,
  mut ws: WebSocketServer<Option<NegotiatedFlate2>, Xorshift64, TcpStream, &mut WebSocketBuffer>,
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
