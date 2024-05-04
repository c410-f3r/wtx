//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use tokio::net::TcpListener;
use wtx::{
  misc::TokioRustlsAcceptor,
  web_socket::{FrameBufferVec, WebSocketBuffer},
};

static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
static KEY: &[u8] = include_bytes!("../../.certs/key.pem");

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  let acceptor = TokioRustlsAcceptor::default().with_cert_chain_and_priv_key(CERT, KEY).unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let local_acceptor = acceptor.clone();
    let _jh = tokio::spawn(async move {
      let tls_stream = local_acceptor.accept(stream).await.unwrap();
      common::_accept_conn_and_echo_frames(
        (),
        &mut FrameBufferVec::default(),
        tls_stream,
        &mut WebSocketBuffer::default(),
      )
      .await
      .unwrap();
    });
  }
}
