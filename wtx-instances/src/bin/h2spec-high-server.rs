//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::{simple_seed, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  LowLevelServer::tokio_high_http2(
    "127.0.0.1:9000",
    || Ok(((), Http2Buffer::new(Xorshift64::from(simple_seed())), Http2Params::default())),
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
  req.rrd.body.extend_from_slice(b"Hello")?;
  Ok(req.into_response(StatusCode::Ok))
}
