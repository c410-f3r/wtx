//! WebSocket CLI client.

#[path = "./common/mod.rs"]
mod common;

use std::{io::Cursor, sync::Arc};
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
};
use tokio_rustls::{
  rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName},
  TlsConnector,
};
use webpki_roots::TLS_SERVER_ROOTS;
use wtx::{
  rng::StdRng,
  web_socket::{
    handshake::{WebSocketConnect, WebSocketConnectRaw},
    FrameBufferVec, FrameMutVec, OpCode,
  },
  UriParts,
};

static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

#[tokio::main(flavor = "current_thread")]
async fn main() -> wtx::Result<()> {
  let fb = &mut FrameBufferVec::default();
  let pb = &mut <_>::default();
  let uri = common::_uri_from_args();
  let uri_parts = UriParts::from(uri.as_str());
  let (_, mut ws) = WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut <_>::default(),
    pb,
    rng: StdRng::default(),
    stream: tls_connector()?
      .connect(
        ServerName::try_from(uri_parts.hostname).map_err(|_err| wtx::Error::MissingHost)?,
        TcpStream::connect(uri_parts.host).await?,
      )
      .await?,
    uri: &uri,
  }
  .connect()
  .await?;
  let mut buffer = String::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame(fb) => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => println!("{elem}"),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = reader.read_line(&mut buffer) => {
        let _ = read_rslt?;
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, buffer.as_bytes())?).await?;
      }
    }
  }
  Ok(())
}

// You probably shouldn't use self-signed root authorities in a production environment.
fn tls_connector() -> wtx::Result<TlsConnector> {
  let mut root_store = RootCertStore::empty();
  root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
    OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
  }));
  let _ = root_store.add_parsable_certificates(&rustls_pemfile::certs(&mut Cursor::new(ROOT_CA))?);
  let config = ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();
  Ok(TlsConnector::from(Arc::new(config)))
}
