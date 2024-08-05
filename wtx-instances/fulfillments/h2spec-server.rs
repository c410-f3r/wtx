//! h2spec

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  rng::StdRng,
};

#[tokio::main]
async fn main() {
  LowLevelServer::tokio_http2(
    (),
    "127.0.0.1:9000",
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

async fn handle(
  (_, mut req): ((), Request<ReqResBuffer>),
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  req.rrd.extend_body(b"Hello").unwrap();
  Ok(req.into_response(StatusCode::Ok))
}
