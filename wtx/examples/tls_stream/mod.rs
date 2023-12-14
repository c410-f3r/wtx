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

static ROOT_CA: &[u8] = include_bytes!("../../../.certs/root-ca.crt");

pub(crate) async fn _tls_stream_host(host: &str, hostname: &str) -> TlsStream<TcpStream> {
  let stream = TcpStream::connect(host).await.unwrap();
  _tls_connector()
    .connect(ServerName::try_from(hostname.to_string()).unwrap(), stream)
    .await
    .unwrap()
}

pub(crate) async fn _tls_stream_stream(hostname: &str, stream: TcpStream) -> TlsStream<TcpStream> {
  _tls_connector()
    .connect(ServerName::try_from(hostname.to_string()).unwrap(), stream)
    .await
    .unwrap()
}

// You probably shouldn't use self-signed root authorities in a production environment.
fn _tls_connector() -> TlsConnector {
  let mut root_store = RootCertStore::empty();
  root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
  let certs: Vec<_> = certs(&mut &*ROOT_CA).collect::<Result<_, _>>().unwrap();
  let _ = root_store.add_parsable_certificates(certs);
  let config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();
  TlsConnector::from(Arc::new(config))
}
