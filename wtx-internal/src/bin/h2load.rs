//! h2load

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  http::{
    AutoStream, HttpRecvParams, ManualServerStream, OperationMode, OptionedServer, ReqResBuffer,
    Response, StatusCode,
  },
  http2::Http2Buffer,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::http2_tokio(
    ((), "127.0.0.1:9000", (), ()),
    |_| Ok(()),
    |_, stream| async move {
      stream.set_nodelay(true).unwrap();
      Ok(stream.into_split())
    },
    |_conn_error| {},
    |_, mut rng| Ok(((), Http2Buffer::new(&mut rng), HttpRecvParams::with_permissive_params())),
    |_| Ok(()),
    |_, _, _, _, _| Ok(((), OperationMode::Auto)),
    |_stream_error| {},
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
