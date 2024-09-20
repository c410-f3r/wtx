//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::StdRng,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = wtx::misc::tracing_tree_init(None);
  LowLevelServer::tokio_http2(
    "127.0.0.1:9000",
    || Ok(((), Http2Buffer::new(StdRng::default()), Http2Params::default())),
    |error| eprintln!("{error}"),
    handle,
    || Ok(((), ReqResBuffer::empty())),
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
  )
  .await
}

async fn handle(
  _: (),
  _: (),
  mut req: Request<ReqResBuffer>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  req.rrd.data.extend_from_slice(b"Hello")?;
  Ok(req.into_response(StatusCode::Ok))
}
