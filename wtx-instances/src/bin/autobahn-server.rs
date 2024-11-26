//! WebSocket autobahn server.

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::TcpStream;
use wtx::{
  http::OptionedServer,
  web_socket::{
    compression::{Flate2, NegotiatedFlate2},
    OpCode, WebSocketBuffer, WebSocketServer,
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::web_socket_tokio(
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
  mut ws: WebSocketServer<Option<NegotiatedFlate2>, TcpStream, &mut WebSocketBuffer>,
) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.parts_mut();
  loop {
    let mut frame = reader.read_frame(&mut common).await?;
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
