//! h2load

#![expect(clippy::print_stderr, reason = "internal")]

use std::u32;

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{Headers, OptionedServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params, ServerStreamTokio},
  misc::{simple_seed, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::tokio_high_http2(
    "127.0.0.1:9000",
    auto,
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
    manual,
    || Ok(((), ReqResBuffer::empty())),
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
  )
  .await
}
async fn auto(
  _: (),
  _: (),
  mut req: Request<ReqResBuffer>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  Ok(req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  _: (),
  _: Headers,
  _: ServerStreamTokio<Http2Buffer, OwnedWriteHalf, false>,
) -> Result<(), wtx::Error> {
  Ok(())
}
