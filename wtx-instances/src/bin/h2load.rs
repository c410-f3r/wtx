//! h2load

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{
    AutoStream, ManualServerStreamTokio, OperationMode, OptionedServer, ReqResBuffer, Response,
    StatusCode,
  },
  http2::{Http2Buffer, Http2Params},
  misc::{simple_seed, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::http2_tokio(
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
    |_, _, _, _| Ok(((), OperationMode::Auto)),
    || Ok(((), ReqResBuffer::empty())),
    (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
  )
  .await
}
async fn auto(_: (), mut ha: AutoStream<(), ()>) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.rrd.clear();
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  _: ManualServerStreamTokio<(), Http2Buffer, (), OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
