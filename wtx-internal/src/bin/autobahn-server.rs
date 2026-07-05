//! WebSocket autobahn server.

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  http::WebSocketServerFramework,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModePlainText},
  web_socket::{
    OpCode, WebSocket, WebSocketPayloadOrigin,
    web_socket_compression::{NegotiatedZlibRs, ZlibRs},
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  WebSocketServerFramework::tokio(ChaCha20::from_std_random()?, TlsConfig::plaintext().into())?
    .set_compression(ZlibRs::default())
    .set_error_cb(|error| eprintln!("{error}"))
    .run("127.0.0.1:9070", (("/", echo),))
    .await
}

async fn echo(
  mut buffer: Vector<u8>,
  mut ws: WebSocket<Option<NegotiatedZlibRs>, TcpStream, TlsModePlainText, false>,
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
