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
  tls::{TlsConfig, TlsConnector},
  web_socket::{Frame, OpCode, WebSocketConnector, WebSocketPartsOwned, WebSocketPayloadOrigin},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_TLS_URI");
  let stream = TlsConnector::default()
    .plain_text()
    .push_cert(wtx_instances::ROOT_CA)
    .connect(&TlsConfig::default(), TcpStream::connect(uri.hostname_with_implied_port()).await?)
    .await?;
  let ws = WebSocketConnector::default().connect(stream, &uri.to_ref()).await?;
  let WebSocketPartsOwned { mut reader, replier, mut writer } = ws.into_parts(tokio::io::split)?;

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
