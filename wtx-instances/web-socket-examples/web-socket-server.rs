//! Serves requests using low-level WebSockets resources along side self-made certificates.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  http::OptionedServer,
  rng::{ChaCha20, SeedableRng, Xorshift64},
  tls::{TlsAcceptor, TlsBuffer, TlsConfig, TlsModeVerifyFull, TlsStream},
  web_socket::{OpCode, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::web_socket_tokio(
    &wtx_instances::host_from_args(),
    None,
    ChaCha20::from_std_random()?,
    || {},
    |error| eprintln!("{error}"),
    handle,
    |stream| async move {
      Ok(
        TlsAcceptor::default()
          .push_cert(wtx_instances::CERT)
          .push_priv_key(wtx_instances::KEY)
          .accept(stream, TlsBuffer::default(), &TlsConfig::default())
          .await?,
      )
    },
  )
  .await
}

async fn handle(
  mut ws: WebSocket<
    (),
    Xorshift64,
    TlsStream<TcpStream, TlsBuffer, TlsModeVerifyFull, false>,
    &mut WebSocketBuffer,
    false,
  >,
) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  let mut buffer = Vector::new();
  loop {
    let mut frame =
      reader.read_frame(&mut buffer, &mut common, WebSocketPayloadOrigin::Adaptive).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        writer.write_frame(&mut common, &mut frame).await?;
      }
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}
