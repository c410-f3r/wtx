//! WebSocket CLI client.

mod common;

use std::{
    io::{self, ErrorKind},
    sync::Arc,
};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName},
    TlsConnector,
};
use webpki_roots::TLS_SERVER_ROOTS;
use wtx::{web_socket::OpCode, UriParts};

#[tokio::main]
async fn main() -> wtx::Result<()> {
    let fb = &mut <_>::default();
    let map_err = |_err| io::Error::new(ErrorKind::InvalidInput, "invalid dnsname");
    let rb = &mut <_>::default();
    let uri = common::_uri_from_args();
    let uri_parts = UriParts::from(uri.as_str());
    let mut ws = common::_connect(
        fb,
        &uri,
        rb,
        tls_connector()
            .connect(
                ServerName::try_from(uri_parts.hostname).map_err(map_err)?,
                TcpStream::connect(uri_parts.host).await?,
            )
            .await?,
    )
    .await?;

    loop {
        let frame = ws.read_msg(fb).await?;
        match (frame.op_code(), frame.text_payload()) {
            (_, Some(elem)) => println!("{elem}"),
            (OpCode::Close, _) => break,
            _ => {}
        }
    }
    Ok(())
}

fn tls_connector() -> TlsConnector {
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(config))
}
