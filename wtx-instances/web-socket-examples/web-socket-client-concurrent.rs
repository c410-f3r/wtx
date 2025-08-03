//! Encrypted WebSocket client that reads and writes frames in different tasks.
//!
//! Replies aren't automatically handled by the system in concurrent scenarios because there are
//! multiple ways to synchronize resources. In this example, `Close` and `Ping` frames are manually
//! managed using a `Mutex` but you can also utilize channels or any other method.

extern crate tokio;
extern crate tokio_rustls;
extern crate wtx;
extern crate wtx_instances;

use tokio::{
  io::{ReadHalf, WriteHalf},
  net::TcpStream,
  sync::Mutex,
};
use tokio_rustls::client::TlsStream;
use wtx::{
  collection::Vector,
  misc::{TokioRustlsConnector, Uri},
  rng::Xorshift64,
  sync::Arc,
  web_socket::{
    Frame, OpCode, WebSocketConnector, WebSocketReaderPartOwned, WebSocketWriterPartOwned,
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
  let parts = ws.into_parts(tokio::io::split)?;
  let writer_for_reader = Arc::new(Mutex::new(parts.writer));
  let writer_for_writer = writer_for_reader.clone();
  let tuple = tokio::join!(reader(parts.reader, writer_for_reader), writer(writer_for_writer));
  tuple.0?;
  tuple.1?;
  Ok(())
}

async fn reader(
  mut reader: WebSocketReaderPartOwned<(), Xorshift64, ReadHalf<TlsStream<TcpStream>>, true>,
  writer: Arc<
    Mutex<WebSocketWriterPartOwned<(), Xorshift64, WriteHalf<TlsStream<TcpStream>>, true>>,
  >,
) -> wtx::Result<()> {
  let mut buffer = Vector::new();
  loop {
    let frame = reader.read_frame(&mut buffer).await?.0;
    match (frame.op_code(), frame.text_payload()) {
      (OpCode::Close, Some(text)) => {
        println!("Received close frame: {text}");
        writer.lock().await.write_frame(&mut Frame::new_fin(OpCode::Close, [])).await?;
        break;
      }
      (OpCode::Ping, _) => {
        writer.lock().await.write_frame(&mut Frame::new_fin(OpCode::Pong, [])).await?;
      }
      (_, text) => {
        if let Some(elem) = text {
          println!("Received text frame: {elem}")
        }
      }
    }
  }
  Ok(())
}

async fn writer(
  writer: Arc<
    Mutex<WebSocketWriterPartOwned<(), Xorshift64, WriteHalf<TlsStream<TcpStream>>, true>>,
  >,
) -> wtx::Result<()> {
  writer.lock().await.write_frame(&mut Frame::new_fin(OpCode::Close, *b"Hi and Bye")).await?;
  Ok(())
}
