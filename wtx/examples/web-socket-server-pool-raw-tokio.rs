//! Uses a pool of resources to avoid having to heap-allocate for every new connection.
//!
//! Semantically speaking, this WebSocket code only accepts a connection and then immediately
//! closes it.

#[path = "./common/mod.rs"]
mod common;

use std::sync::OnceLock;
use tokio::{net::TcpListener, sync::MappedMutexGuard};
use wtx::{
  pool_manager::{Pool as _, ResourceManager, StaticPoolTokioMutex, WebSocketRM},
  rng::StaticRng,
  web_socket::{
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    FrameMutVec, OpCode,
  },
};

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _jh = tokio::spawn(async move {
      let (fb, wsb) = &mut *pool_resource().await;
      let mut ws = WebSocketAcceptRaw { compression: (), rng: StaticRng::default(), stream, wsb }
        .accept(|_| true)
        .await
        .unwrap();
      ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap()).await.unwrap();
    });
  }
}

async fn pool_resource() -> MappedMutexGuard<'static, <WebSocketRM as ResourceManager>::Resource> {
  static POOL: OnceLock<
    StaticPoolTokioMutex<<WebSocketRM as ResourceManager>::Resource, WebSocketRM, 8>,
  > = OnceLock::new();
  POOL
    .get_or_init(|| StaticPoolTokioMutex::new(WebSocketRM::web_socket_rm()).unwrap())
    .get()
    .await
    .unwrap()
}
