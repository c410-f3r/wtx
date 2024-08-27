//! h2spec

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::StdRng,
};

#[tokio::main]
async fn main() {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = wtx::misc::tracing_tree_init();
  LowLevelServer::tokio_http2(
    (),
    "127.0.0.1:9000",
    |err| eprintln!("Error: {err:?}"),
    handle,
    || Ok(Http2Buffer::new(StdRng::default())),
    Http2Params::default,
    || Ok(ReqResBuffer::default()),
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
  )
  .await
  .unwrap();
}

async fn handle(
  mut req: Request<ReqResBuffer>,
  _: (),
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  req.rrd.data.extend_from_slice(b"Hello").unwrap();
  Ok(req.into_response(StatusCode::Ok))
}
