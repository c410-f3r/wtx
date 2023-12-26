use rustls_pemfile::certs;
use rustls_pki_types::ServerName;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{
  client::TlsStream,
  rustls::{ClientConfig, RootCertStore},
  TlsConnector,
};
use webpki_roots::TLS_SERVER_ROOTS;

/// Private method.
pub async fn tls_stream_from_host(
  host: &str,
  hostname: &str,
  root_ca: Option<&[u8]>,
) -> crate::Result<TlsStream<TcpStream>> {
  let stream = TcpStream::connect(host).await?;
  Ok(tls_connector(root_ca)?.connect(server_name(hostname)?, stream).await?)
}

/// Private method.
pub async fn tls_stream_from_stream(
  hostname: &str,
  root_ca: Option<&[u8]>,
  stream: TcpStream,
) -> crate::Result<TlsStream<TcpStream>> {
  Ok(tls_connector(root_ca)?.connect(server_name(hostname)?, stream).await?)
}

fn server_name(hostname: &str) -> crate::Result<ServerName<'static>> {
  Ok(
    ServerName::try_from(hostname.to_string())
      .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?,
  )
}

fn tls_connector(root_ca: Option<&[u8]>) -> crate::Result<TlsConnector> {
  let mut root_store = RootCertStore::empty();
  root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
  if let Some(elem) = root_ca {
    let certs: Vec<_> = certs(&mut &*elem).collect::<Result<_, _>>()?;
    let _ = root_store.add_parsable_certificates(certs);
  }
  let config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();
  Ok(TlsConnector::from(Arc::new(config)))
}
