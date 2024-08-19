//! Serves requests using low-level WebSockets resources along side self-made certificates.

extern crate tokio;
extern crate tokio_rustls;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use wtx::{
  http::LowLevelServer,
  misc::{StdRng, TokioRustlsAcceptor},
  web_socket::{FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServer},
};

#[tokio::main]
async fn main() {
  LowLevelServer::tokio_web_socket(
    &wtx_instances::host_from_args(),
    None,
    || {},
    |err| eprintln!("Connection error: {err:?}"),
    handle,
    (
      || {
        TokioRustlsAcceptor::without_client_auth()
          .build_with_cert_chain_and_priv_key(wtx_instances::CERT, wtx_instances::KEY)
          .unwrap()
      },
      |acceptor| acceptor.clone(),
      |acceptor, stream| async move { Ok(acceptor.accept(stream).await?) },
    ),
  )
  .await
  .unwrap()
}

async fn handle(
  (fb, mut ws): (
    &mut FrameBufferVec,
    WebSocketServer<(), StdRng, TlsStream<TcpStream>, &mut WebSocketBuffer>,
  ),
) -> wtx::Result<()> {
  loop {
    let mut frame = ws.read_frame(fb).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        ws.write_frame(&mut frame).await?;
      }
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}
