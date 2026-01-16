//! WebSocket autobahn server.

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  http::OptionedServer,
  rng::{ChaCha20, SeedableRng, Xorshift64},
  web_socket::{
    OpCode, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin,
    compression::{Flate2, NegotiatedFlate2},
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::web_socket_tokio(
    "127.0.0.1:9070",
    None,
    ChaCha20::from_std_random()?,
    Flate2::default,
    |error| eprintln!("{error}"),
    handle,
    |stream| async move { Ok(stream) },
  )
  .await
}

async fn handle(
  mut ws: WebSocket<Option<NegotiatedFlate2>, Xorshift64, TcpStream, &mut WebSocketBuffer, false>,
) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  let mut buffer = Vector::new();
  loop {
    let mut frame =
      reader.read_frame(&mut buffer, &mut common, WebSocketPayloadOrigin::Adaptive).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        writer.write_frame(&mut common, &mut frame).await?;
      }
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}
