//! Serves requests using low-level HTTP/2 resources along side self-made certificates.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::{StdRng, TokioRustlsAcceptor},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  LowLevelServer::tokio_http2(
    &wtx_instances::host_from_args(),
    || Ok(((), Http2Buffer::new(StdRng::default()), Http2Params::default())),
    |error| eprintln!("{error}"),
    handle,
    || Ok(((), ReqResBuffer::empty())),
    (
      || {
        TokioRustlsAcceptor::without_client_auth()
          .build_with_cert_chain_and_priv_key(wtx_instances::CERT, wtx_instances::KEY)
      },
      |acceptor| acceptor.clone(),
      |acceptor, stream| async move { Ok(tokio::io::split(acceptor.accept(stream).await?)) },
    ),
  )
  .await
}

async fn handle(
  _: (),
  _: (),
  mut req: Request<ReqResBuffer>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  Ok(req.into_response(StatusCode::Ok))
}
