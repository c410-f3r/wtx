//! h2load

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{
    AutoStream, ManualServerStreamTokio, OptionedServer, ReqResBuffer, Response, StatusCode,
    StreamMode,
  },
  http2::{Http2Buffer, Http2Params},
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
    |_, _, _| Ok(StreamMode::Auto),
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
  )
  .await
}
async fn auto(mut ha: AutoStream<(), ()>) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.rrd.clear();
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: ManualServerStreamTokio<(), (), Http2Buffer, OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
