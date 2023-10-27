//! Uses a pool of resources to avoid having to heap-allocate bytes for every new connection. This
//! approach also imposes an upper bound on the number of concurrent processing requests.
//!
//! Semantically speaking, the WebSocket code only accepts a connection and then immediately
//! closes it.

#[path = "./common/mod.rs"]
mod common;

use deadpool::unmanaged::{Object, Pool};
use std::sync::OnceLock;
use tokio::net::TcpListener;
use wtx::{
  rng::StaticRng,
  web_socket::{
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    FrameBufferVec, FrameMutVec, OpCode,
  },
  PartitionedBuffer,
};

const POOL_LEN: usize = 32;

static POOL: OnceLock<Pool<(FrameBufferVec, PartitionedBuffer)>> = OnceLock::new();

#[tokio::main(flavor = "current_thread")]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(common::_host_from_args()).await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      let fun = || async {
        let (fb, pb) = &mut *pool_elem().await?;
        let mut ws = WebSocketAcceptRaw {
          compression: (),
          key_buffer: &mut <_>::default(),
          pb,
          rng: StaticRng::default(),
          stream,
        }
        .accept(|_| true)
        .await?;
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[])?).await?;
        wtx::Result::Ok(())
      };
      if let Err(err) = fun().await {
        println!("{err}");
      }
    });
  }
}

async fn pool_elem() -> wtx::Result<Object<(FrameBufferVec, PartitionedBuffer)>> {
  Ok(POOL.get_or_init(|| Pool::from((0..POOL_LEN).map(|_| <_>::default()))).get().await?)
}
