//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use std::u32;

use wtx::{
  http::{LowLevelServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params},
  misc::{simple_seed, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  LowLevelServer::tokio_high_http2(
    "127.0.0.1:9000",
    || {
      Ok((
        (),
        Http2Buffer::new(Xorshift64::from(simple_seed())),
        Http2Params::default()
          .set_max_concurrent_streams_num(u32::MAX)
          .set_max_recv_streams_num(u32::MAX),
      ))
    },
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
  req: Request<ReqResBuffer>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  Ok(req.into_response(StatusCode::Ok))
}
