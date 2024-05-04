macro_rules! call_tests {
  (($ty:ident, $http2:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$ty($http2).await;
      tokio::time::sleep(Duration::from_millis(200)).await;
    )+
  };
}

use crate::{
  http2::{Http2Buffer, Http2Params, Http2Tokio},
  misc::_uri,
  rng::StaticRng,
};
use core::{
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};
use std::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[tokio::test]
async fn connections() {
  let uri = _uri();

  let listener = TcpListener::bind(uri.host()).await.unwrap();
  let _server_jh = tokio::spawn(async move {
    let (stream, _) = listener.accept().await.unwrap();
    let mut server =
      Http2Tokio::accept(Http2Buffer::new(StaticRng::default()), Http2Params::default(), stream)
        .await
        .unwrap();
    call_tests!((server, &mut server), Stub);
    HAS_SERVER_FINISHED.store(true, Ordering::Relaxed);
  });

  let mut client = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host().to_socket_addrs().unwrap().next().unwrap()).await.unwrap(),
  )
  .await
  .unwrap();
  call_tests!((client, &mut client), Stub);

  let mut has_server_finished = false;
  for _ in 0..15 {
    let local_has_server_finished = HAS_SERVER_FINISHED.load(Ordering::Relaxed);
    if local_has_server_finished {
      has_server_finished = local_has_server_finished;
      break;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
  }
  if !has_server_finished {
    panic!("Server didn't finish");
  }
}

trait Test {
  async fn client(http2: &mut Http2Tokio<Http2Buffer<true>, TcpStream, true>);

  async fn server(http2: &mut Http2Tokio<Http2Buffer<false>, TcpStream, false>);
}

struct Stub;
impl Test for Stub {
  async fn client(_: &mut Http2Tokio<Http2Buffer<true>, TcpStream, true>) {}

  async fn server(_: &mut Http2Tokio<Http2Buffer<false>, TcpStream, false>) {}
}
