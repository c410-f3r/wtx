//! High-level WebSocket server that supports the routing of paths.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  http::WebSocketServerFramework,
  misc::SecretContext,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
  web_socket::{OpCode, WebSocket, WebSocketPayloadOrigin},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

type LocalWebSocket = WebSocket<(), TcpStream, TlsModeVerified, false>;

fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;
  let tls_config = TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    &mut rng,
    (secret_context, &mut SECRET_KEY.clone()),
  )?;
  let router = (("/echo", echo),);
  WebSocketServerFramework::tokio(tls_config)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
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
