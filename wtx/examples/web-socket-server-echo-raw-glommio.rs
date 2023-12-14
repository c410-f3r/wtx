//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

#[cfg(feature = "async-send")]
fn main() {}

#[cfg(not(feature = "async-send"))]
fn main() {
  use glommio::{net::TcpListener, LocalExecutorBuilder};

  LocalExecutorBuilder::default()
    .spawn::<_, _, ()>(|| async {
      let listener = TcpListener::bind(crate::common::_host_from_args()).unwrap();
      loop {
        let stream = listener.accept().await.unwrap();
        let _jh = glommio::spawn_local(async move {
          crate::common::_accept_conn_and_echo_frames((), &mut <_>::default(), stream)
            .await
            .unwrap();
        })
        .detach();
      }
    })
    .unwrap()
    .join()
    .unwrap();
}
