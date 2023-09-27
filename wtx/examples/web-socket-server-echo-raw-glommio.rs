//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

fn main() -> wtx::Result<()> {
  use glommio::{net::TcpListener, LocalExecutorBuilder};

  LocalExecutorBuilder::default()
    .spawn::<_, _, wtx::Result<_>>(|| async {
      let listener = TcpListener::bind(crate::common::_host_from_args())?;
      loop {
        let stream = listener.accept().await?;
        let _jh = glommio::spawn_local(async move {
          let fb = &mut <_>::default();
          let pb = &mut <_>::default();
          if let Err(err) = crate::common::_accept_conn_and_echo_frames((), fb, pb, stream).await {
            println!("{err}");
          }
        })
        .detach();
      }
    })?
    .join()??;
  Ok(())
}
