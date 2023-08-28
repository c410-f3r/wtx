//! WebSocket echo server.

mod common;

use rustls_pemfile::{certs, pkcs8_private_keys};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};

static CERT: &[u8] = include_bytes!("./localhost.crt");
static KEY: &[u8] = include_bytes!("./localhost.key");

#[tokio::main]
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

fn tls_acceptor() -> wtx::Result<TlsAcceptor> {
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut &*KEY)
        .map(|certs| certs.into_iter().map(PrivateKey).collect())
        .map_err(wtx::Error::from)?;
    let certs = certs(&mut &*CERT)
        .map(|certs| certs.into_iter().map(Certificate).collect())
        .map_err(wtx::Error::from)?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
