//! Encrypted WebSocket client that reads and writes frames in different tasks.
//!
//! Replies aren't automatically handled by the system in concurrent scenarios because there are
//! multiple ways to synchronize resources. In this example, reply frames are managed in the same
//! task but you can also utilize any other method.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  misc::Uri,
  rng::{ChaCha20, SeedableRng as _},
  tls::{TlsConfig, TlsConnector},
  web_socket::{Frame, OpCode, WebSocketConnector, WebSocketPartsOwned, WebSocketPayloadOrigin},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut rng = ChaCha20::from_getrandom()?;
  let tls_stream = TlsConnector::default()
    .connect(
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port()).await?,
      &TlsConfig::default(),
    )
    .await?;
  let ws = WebSocketConnector::default().connect(tls_stream, &uri.to_ref()).await?;
  let parts = ws.into_split(|inner_tls_stream| {
    inner_tls_stream.into_split(|inner_tcp_stream| inner_tcp_stream.into_split())
  })?;
  let WebSocketPartsOwned { mut reader, replier, mut writer } = parts;
  let reader_fut = async {
    let mut buffer = Vector::new();
    loop {
      let frame = reader.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await?;
      match (frame.op_code(), frame.text_payload()) {
        // A special version of this frame has already been sent to the replier
        (OpCode::Close, _) => break,
        // A `Pong` frame with the same content has already been sent to the replier
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
    writer.write_frame(&mut Frame::new_fin(OpCode::Close, *b"Bye")).await?;
    loop {
      let mut control_frame = replier.reply_frame().await;
      if writer.write_reply_frame(&mut control_frame).await? {
        break;
      }
    }
    wtx::Result::Ok(())
  };

  let (reader_rslt, writer_rslt) = tokio::join!(reader_fut, writer_fut);
  reader_rslt?;
  writer_rslt?;
  Ok(())
}
