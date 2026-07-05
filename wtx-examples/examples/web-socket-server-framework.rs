//! High-level WebSocket server that supports the routing of paths.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  http::WebSocketServerFramework,
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsModeVerified},
  web_socket::{OpCode, WebSocket, WebSocketPayloadOrigin},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

type LocalWebSocket = WebSocket<(), TcpStream, TlsModeVerified, false>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  WebSocketServerFramework::tokio(
    ChaCha20::from_getrandom()?,
    TlsConfig::from_keys_pem(
      TlsModeVerified::default(),
      PUBLIC_KEY.try_into()?,
      SECRET_KEY.try_into()?,
    )?,
  )?
  .run(&host_from_args(), (("/echo", echo),))
  .await
}

async fn echo(mut buffer: Vector<u8>, mut ws: LocalWebSocket) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  loop {
    let origin = WebSocketPayloadOrigin::Adaptive;
    let mut frame = reader.read_frame(&mut buffer, &mut common, origin).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
      OpCode::Close => return Ok(()),
      _ => {}
    }
  }
}
