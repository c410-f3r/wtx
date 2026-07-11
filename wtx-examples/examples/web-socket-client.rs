//! WebSocket client that reads and writes frames in the same task.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use wtx::{
  collections::Vector,
  misc::Uri,
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsConnectorBuilder, TlsModeVerified},
  web_socket::{OpCode, WebSocketConnector, WebSocketPayloadOrigin},
};
use wtx_examples::{ROOT_CA, uri_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new(uri_from_args());
  let tls_connector = TlsConnectorBuilder::tokio(uri)
    .build(
      TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
      ChaCha20::from_getrandom()?,
    )
    .await?;
  let mut ws = WebSocketConnector::default().connect(tls_connector).await?;
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
