//! WebSocket CLI client that enables real-time communication by allowing users to send and
//! receive messages through typing.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  misc::Uri,
  rng::{ChaCha20, SeedableRng},
  tls::{TlsConfig, TlsConnector},
  web_socket::{OpCode, WebSocketConnector, WebSocketPayloadOrigin},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut rng = ChaCha20::from_std_random()?;
  let tls_stream = TlsConnector::default()
    .connect(
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port()).await?,
      &TlsConfig::default(),
    )
    .await?;
  let mut ws = WebSocketConnector::default()
    .headers([("custom-key", "custom value")]) // Headers are optional. This method can be omitted.
    .connect(tls_stream, &uri.to_ref())
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
