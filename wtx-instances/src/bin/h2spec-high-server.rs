//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{
    AutoStream, ManualServerStreamTokio, OperationMode, OptionedServer, ReqResBuffer, Response,
    StatusCode,
  },
  http2::{Http2Buffer, Http2Params},
  misc::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::http2_tokio(
    "127.0.0.1:9000",
    auto,
    || Ok(((), Http2Buffer::new(Xorshift64::from(simple_seed())), Http2Params::default())),
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
  ha.req.rrd.body.extend_from_copyable_slice(b"Hello")?;
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  _: ManualServerStreamTokio<(), Http2Buffer, (), OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
