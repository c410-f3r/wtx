//! h2spec

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
  OptionedServer::http2_tokio(
    "127.0.0.1:9000",
    auto,
    || Ok(((), Http2Buffer::new(Xorshift64::from(simple_seed())), Http2Params::default())),
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
  ha.req.rrd.body.extend_from_copyable_slice(b"Hello")?;
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: ManualServerStreamTokio<(), Http2Buffer, (), (), OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
