//! h2load

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
    ((), "127.0.0.1:9000", Xorshift64::from(simple_seed()), ()),
    |_| Ok(()),
    |_, stream| async move {
      stream.set_nodelay(true)?;
      Ok(stream.into_split())
    },
    |error| eprintln!("{error}"),
    |mut rng| {
      Ok((
        (),
        Http2Buffer::new(&mut rng),
        Http2Params::default()
          .set_max_concurrent_streams_num(u32::MAX)
          .set_max_recv_streams_num(u32::MAX),
      ))
    },
    |_| Ok(()),
    |_, _, _, _, _| Ok(((), OperationMode::Auto)),
    |error| eprintln!("{error}"),
    auto,
    manual,
  )
  .await
}

async fn auto(_: (), mut ha: AutoStream<(), ()>) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.clear();
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  _: ManualServerStream<(), Http2Buffer, (), OwnedWriteHalf>,
) -> Result<(), wtx::Error> {
  Ok(())
}
