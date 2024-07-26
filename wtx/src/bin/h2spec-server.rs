//! h2spec

use wtx::{
  http::{server::OptionedServer, ReqResBuffer, Request, StatusCode},
  http2::{Http2Buffer, Http2Params},
  rng::StdRng,
};

#[tokio::main]
async fn main() {
  OptionedServer::tokio_http2(
    "127.0.0.1:9000".parse().unwrap(),
    |err| eprintln!("Error: {err:?}"),
    handle,
    || Ok(Http2Buffer::new(StdRng::default())),
    Http2Params::default,
    || Ok(ReqResBuffer::default()),
    (|| {}, |_| {}, |_, stream| async move { Ok(stream) }),
  )
  .await
  .unwrap()
}

async fn handle(req: &mut Request<&mut ReqResBuffer>) -> Result<StatusCode, wtx::Error> {
  req.rrd.clear();
  req.rrd.extend_body(b"Hello").unwrap();
  Ok(StatusCode::Ok)
}
