//! Encrypted WebSocket client that reads and writes frames in the same task.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  misc::Uri,
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsConnector},
  web_socket::{OpCode, WebSocketConnector, WebSocketPayloadOrigin},
};
use wtx_examples::LocalTlsMode;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("https://c410-f3r.github.io/wtx");
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let mut ws = WebSocketConnector::default()
    .connect(
      &mut ChaCha20::from_getrandom()?,
      TlsConnector::from_stream(stream).tls_mode(LocalTlsMode::default()),
      &TlsConfig::from_ccadb(),
      &uri.to_ref(),
    )
    .await?;
  let mut buffer = Vector::new();
  loop {
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await?;
    match (frame.op_code(), frame.text_payload()) {
      // `read_frame` internally already sent a Close response
      (OpCode::Close, _) => {
        break;
      }
      // `read_frame` internally already sent a Pong response
      (OpCode::Ping, _) => {}
      // For any other type, `read_frame` doesn't automatically send frames
      (_, text) => {
        if let Some(elem) = text {
          println!("Received text frame: {elem}")
        }
      }
    }
  }
  Ok(())
}
