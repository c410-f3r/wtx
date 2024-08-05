//! Http2 echo server.

#[path = "./common/mod.rs"]
mod common;

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::TokioRustlsAcceptor,
  rng::StdRng,
};

static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
static KEY: &[u8] = include_bytes!("../../.certs/key.pem");

#[tokio::main]
async fn main() {
  LowLevelServer::tokio_http2(
    (),
    &common::_host_from_args(),
    |err| eprintln!("Error: {err:?}"),
    handle,
    || Ok(Http2Buffer::new(StdRng::default())),
    || Http2Params::default(),
    || Ok(ReqResBuffer::default()),
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
  (_, mut req): ((), Request<ReqResBuffer>),
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  Ok(req.into_response(StatusCode::Ok))
}
