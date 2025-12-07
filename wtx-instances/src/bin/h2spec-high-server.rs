//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{
    AutoStream, ManualServerStream, OperationMode, OptionedServer, ReqResBuffer, Response,
    StatusCode,
  },
  http2::{Http2Buffer, Http2Params},
  rng::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::http2_tokio(
    ((), "127.0.0.1:9000", (), ()),
    |_| Ok(()),
    |_, stream| async move { Ok(stream.into_split()) },
    |error| eprintln!("{error}"),
    |_| Ok(((), Http2Buffer::new(&mut Xorshift64::from(simple_seed())), Http2Params::default())),
    |_| Ok(((), ReqResBuffer::empty())),
    |_, _, _, _, _| Ok(((), OperationMode::Auto)),
    |error| eprintln!("{error}"),
    auto,
    manual,
  )
  .await
}

async fn auto(_: (), mut ha: AutoStream<(), ()>) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.clear();
  ha.req.rrd.body.extend_from_copyable_slice(b"Hello")?;
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  _: ManualServerStream<(), Http2Buffer, (), OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
