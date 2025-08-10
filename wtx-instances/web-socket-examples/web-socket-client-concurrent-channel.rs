//! Encrypted WebSocket client that reads and writes frames in different tasks.
//!
//! Replies aren't automatically handled by the system in concurrent scenarios because there are
//! multiple ways to synchronize resources. In this example, reply frames are managed using a
//! channel but you can also utilize any other method.

extern crate tokio;
extern crate tokio_rustls;
extern crate wtx;
extern crate wtx_instances;

use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use wtx::{
  collection::Vector,
  misc::{TokioRustlsConnector, Uri, into_rslt},
  web_socket::{
    Frame, FrameVector, OpCode, WebSocketConnector, WebSocketPartsOwned, WebSocketReadMode,
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_TLS_URI");
  let tls_connector = TokioRustlsConnector::from_auto()?.push_certs(wtx_instances::ROOT_CA)?;
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let ws = WebSocketConnector::default()
    .connect(
      tls_connector.connect_without_client_auth(uri.hostname(), stream).await?,
      &uri.to_ref(),
    )
    .await?;
  let WebSocketPartsOwned { mut reader, replier, mut writer } = ws.into_parts(tokio::io::split)?;
  let (sender, mut receiver) = unbounded_channel::<FrameVector<true>>();

  let reader_fut = async {
    let mut buffer = Vector::new();
    loop {
      let frame = reader.read_frame(&mut buffer, WebSocketReadMode::Adaptive).await?;
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

  let sender_fut = async {
    let frame = Frame::new_fin(OpCode::Close, *b"Bye");
    into_rslt(sender.send(frame.to_vector()?).ok())?;
    wtx::Result::Ok(())
  };

  let writer_fut = async {
    loop {
      tokio::select! {
        mut reply_frame_rslt = replier.reply_frame() => {
          if writer.write_reply_frame(&mut reply_frame_rslt).await? {
            break;
          }
        }
        recv_rslt = receiver.recv() => {
          match recv_rslt {
            Some(mut frame) => {
              writer.write_frame(&mut frame).await?;
            },
            None => {
              break;
            },
          }
        }
      }
    }
    wtx::Result::Ok(())
  };

  let (reader_rslt, sender_rslt, writer_rslt) = tokio::join!(reader_fut, sender_fut, writer_fut);
  reader_rslt?;
  sender_rslt?;
  writer_rslt?;
  Ok(())
}
