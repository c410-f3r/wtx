//! WebSocket echo server.

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use wtx::{
  http::LowLevelServer,
  misc::TokioRustlsAcceptor,
  rng::StdRng,
  web_socket::{FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServer},
};

static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
static KEY: &[u8] = include_bytes!("../../.certs/key.pem");

#[tokio::main]
async fn main() {
  LowLevelServer::tokio_web_socket(
    &wtx_instances::host_from_args(),
    None,
    || {},
    |err| eprintln!("Connection error: {err:?}"),
    handle,
    (
      || TokioRustlsAcceptor::default().with_cert_chain_and_priv_key(CERT, KEY).unwrap(),
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
