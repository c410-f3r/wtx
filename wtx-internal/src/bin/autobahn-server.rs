//! WebSocket autobahn server.

#![expect(clippy::print_stderr, reason = "internal")]

use std::net::TcpStream;
use wtx::{
  collection::Vector,
  executor::StdExecutor,
  http::WebSocketServerFramework,
  tls::TlsModePlainText,
  web_socket::{
    OpCode, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin,
    compression::{Flate2, NegotiatedFlate2},
  },
};

#[wtx::main]
async fn main() -> wtx::Result<()> {
  WebSocketServerFramework::new(StdExecutor::default())?
    .set_compression(Flate2::default())
    .set_error_cb(|error| eprintln!("{error}"))
    .set_tls_mode(TlsModePlainText)
    .run("127.0.0.1:9070", (("/", echo),))
    .await
}

async fn echo(
  mut buffer: Vector<u8>,
  mut ws: WebSocket<Option<NegotiatedFlate2>, TcpStream, TlsModePlainText, WebSocketBuffer, false>,
) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  loop {
    let origin = WebSocketPayloadOrigin::Adaptive;
    let mut frame = reader.read_frame(&mut buffer, &mut common, origin).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}
