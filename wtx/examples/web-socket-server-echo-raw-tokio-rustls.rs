//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{io, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{
  rustls::{Certificate, PrivateKey, ServerConfig},
  TlsAcceptor,
};

static CERT: &[u8] = include_bytes!("./cert.pem");
static KEY: &[u8] = include_bytes!("./key.pem");

#[tokio::main(flavor = "current_thread")]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(common::_host_from_args()).await?;
  let tls_acceptor = tls_acceptor()?;
  loop {
    let (stream, _) = listener.accept().await?;
    let local_tls_acceptor = tls_acceptor.clone();
    let _jh = tokio::spawn(async move {
      let fun = || async move {
        let stream = local_tls_acceptor.accept(stream).await?;
        tokio::task::unconstrained(common::_accept_conn_and_echo_frames(
          (),
          &mut <_>::default(),
          &mut <_>::default(),
          stream,
        ))
        .await
      };
      if let Err(err) = fun().await {
        println!("{err}");
      }
    });
  }
}

// You probably shouldn't use self-signed certificates in a production environment.
fn tls_acceptor() -> wtx::Result<TlsAcceptor> {
  let key = pkcs8_private_keys(&mut &*KEY)?
    .into_iter()
    .map(PrivateKey)
    .next()
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No private key"))?;
  let certs: Vec<_> = certs(&mut &*CERT)?.into_iter().map(Certificate).collect();
  let config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(certs, key)?;
  Ok(TlsAcceptor::from(Arc::new(config)))
}
