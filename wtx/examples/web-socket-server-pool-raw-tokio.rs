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
    FrameBufferVec, FrameMutVec, OpCode, WebSocketBuffer,
  },
};

const POOL_LEN: usize = 32;

static POOL: OnceLock<Pool<(FrameBufferVec, WebSocketBuffer)>> = OnceLock::new();

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _jh = tokio::spawn(async move {
      let (fb, wsb) = &mut *pool_elem().await;
      let mut ws = WebSocketAcceptRaw { compression: (), rng: StaticRng::default(), stream, wsb }
        .accept(|_| true)
        .await
        .unwrap();
      ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap()).await.unwrap();
    });
  }
}

async fn pool_elem() -> Object<(FrameBufferVec, WebSocketBuffer)> {
  POOL.get_or_init(|| Pool::from((0..POOL_LEN).map(|_| <_>::default()))).get().await.unwrap()
}
