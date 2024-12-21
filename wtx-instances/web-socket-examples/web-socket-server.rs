//! Serves requests using low-level WebSockets resources along side self-made certificates.

extern crate tokio;
extern crate tokio_rustls;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use wtx::{
  http::OptionedServer,
  misc::TokioRustlsAcceptor,
  web_socket::{OpCode, WebSocket, WebSocketBuffer},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::web_socket_tokio(
    &wtx_instances::host_from_args(),
    None,
    || {},
    |error| eprintln!("{error}"),
    handle,
    (
      || {
        TokioRustlsAcceptor::without_client_auth()
          .build_with_cert_chain_and_priv_key(wtx_instances::CERT, wtx_instances::KEY)
      },
      |acceptor| acceptor.clone(),
      |acceptor, stream| async move { Ok(acceptor.accept(stream).await?) },
    ),
  )
  .await
}

async fn handle(
  mut ws: WebSocket<(), TlsStream<TcpStream>, &mut WebSocketBuffer, false>,
) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.parts_mut();
  loop {
    let mut frame = reader.read_frame(&mut common).await?;
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
