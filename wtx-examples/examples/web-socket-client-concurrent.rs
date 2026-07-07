//! WebSocket client that reads and writes frames in different tasks.
//!
//! Special frames aren't automatically handled by the system in concurrent scenarios because there are
//! multiple ways to synchronize resources. In this example, special frames are managed using a
//! mutex but you can utilize any other method.
//!
//! `wtx-client-concurrent` is an example that uses a channel.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  misc::Uri,
  rng::{ChaCha20, CryptoSeedableRng as _},
  sync::{Arc, AsyncMutex},
  tls::{TlsConfig, TlsConnector, TlsModeVerified},
  web_socket::{Frame, OpCode, WebSocketConnector, WebSocketPayloadOrigin},
};
use wtx_examples::{ROOT_CA, uri_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new(uri_from_args());
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let ws = WebSocketConnector::default()
    .connect(
      TlsConnector::new(
        TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
        ChaCha20::from_getrandom()?,
        stream,
      ),
      &uri.to_ref(),
    )
    .await?;
  let (stream_bridge, mut stream_reader, stream_writer) = ws.into_split()?;
  let stream_writer_bridge = Arc::new(AsyncMutex::new(stream_writer));
  let stream_writer_writer = stream_writer_bridge.clone();

  let bridge_fut = async {
    loop {
      let data = stream_bridge.listen().await;
      if stream_writer_bridge.lock().await.manage_bridge_data(data).await? {
        break;
      }
    }
    wtx::Result::Ok(())
  };

  let reader_fut = async {
    let mut buffer = Vector::new();
    loop {
      let frame = stream_reader.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await?;
      match (frame.op_code(), frame.text_payload()) {
        // A special version of this frame has already been sent to the bridge
        (OpCode::Close, _) => break,
        // A `Pong` frame with the same content has already been sent to the bridge
        (OpCode::Ping, _) => {}
        (_, text) => {
          if let Some(elem) = text {
            println!("Received text frame: {elem}")
          }
        }
      }
    }
    wtx::Result::Ok(())
  };

  let writer_fut = async {
    stream_writer_writer
      .lock()
      .await
      .write_frame(&mut Frame::new_fin(OpCode::Close, *b"Bye")?)
      .await?;
    wtx::Result::Ok(())
  };

  let (bridge_rslt, reader_rslt, writer_rslt) = tokio::join!(bridge_fut, reader_fut, writer_fut);
  bridge_rslt?;
  reader_rslt?;
  writer_rslt?;
  Ok(())
}
