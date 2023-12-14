//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use rustls_pemfile::{certs, pkcs8_private_keys};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
static KEY: &[u8] = include_bytes!("../../.certs/key.pem");

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  let tls_acceptor = tls_acceptor().unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let local_tls_acceptor = tls_acceptor.clone();
    let _jh = tokio::spawn(async move {
      let tls_stream = local_tls_acceptor.accept(stream).await.unwrap();
      common::_accept_conn_and_echo_frames((), &mut <_>::default(), tls_stream).await.unwrap();
    });
  }
}

// You probably shouldn't use self-signed certificates in a production environment.
fn tls_acceptor() -> wtx::Result<TlsAcceptor> {
  let key = pkcs8_private_keys(&mut &*KEY).next().unwrap().map(Into::into).unwrap();
  let certs: Vec<_> = certs(&mut &*CERT).collect::<Result<_, _>>().unwrap();
  let config = ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key).unwrap();
  Ok(TlsAcceptor::from(Arc::new(config)))
}
