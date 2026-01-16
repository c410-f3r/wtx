//! Encrypted WebSocket client that reads and writes frames in different tasks.
//!
//! Replies aren't automatically handled by the system in concurrent scenarios because there are
//! multiple ways to synchronize resources. In this example, reply frames are managed using a
//! channel but you can also utilize any other method.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use wtx::{
  collection::Vector,
  misc::{Uri, into_rslt},
  rng::{ChaCha20, SeedableRng as _},
  tls::{TlsConfig, TlsConnector},
  web_socket::{
    Frame, FrameVector, OpCode, WebSocketConnector, WebSocketPartsOwned, WebSocketPayloadOrigin,
  },
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
  let ws = WebSocketConnector::default().connect(tls_stream, &uri.to_ref()).await?;
  let parts = ws.into_split(|inner_tls_stream| {
    Ok(inner_tls_stream.into_split(|inner_tcp_stream| inner_tcp_stream.into_split()))
  })?;
  let WebSocketPartsOwned { mut reader, replier, mut writer } = parts;
  let (sender, mut receiver) = unbounded_channel::<FrameVector<true>>();

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
