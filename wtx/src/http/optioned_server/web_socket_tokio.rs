use crate::{
  http::{OptionedServer, optioned_server::default_listener},
  misc::FnFut,
  rng::Xorshift64,
  stream::Stream,
  web_socket::{Compression, WebSocket, WebSocketAcceptor, WebSocketBuffer},
};
use alloc::string::String;
use core::fmt::Debug;
use tokio::net::TcpStream;

impl OptionedServer {
  /// Optioned WebSocket server using tokio.
  #[inline]
  pub async fn web_socket_tokio<ACPT, C, E, H, N, S>(
    addr: &str,
    compression_cb: impl Clone + Fn() -> C + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    handle_cb: H,
    (acceptor_cb, net_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> N + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    ACPT: Clone + Send + 'static,
    C: Compression<false> + Send + 'static,
    C::NegotiatedCompression: Send,
    E: Debug + From<crate::Error> + Send + 'static,
    H: Clone
      + FnFut<
        (String, WebSocket<C::NegotiatedCompression, Xorshift64, S, WebSocketBuffer, false>),
        Result = Result<(), E>,
      > + Send
      + 'static,
    N: Send + Future<Output = crate::Result<S>>,
    S: Stream<read(..): Send, write_all(..): Send> + Send,
    <H as FnFut<(
      String,
      WebSocket<C::NegotiatedCompression, Xorshift64, S, WebSocketBuffer, false>,
    )>>::Future: Send,
    for<'handle> &'handle H: Send,
  {
    let acceptor = acceptor_cb()?;
    let listener = default_listener(addr)?;
    loop {
      let conn_acceptor = acceptor.clone();
      let conn_compression_cb = compression_cb.clone();
      let conn_conn_err = err_cb.clone();
      let conn_handle_cb = handle_cb.clone();
      let conn_net_cb = net_cb.clone();
      let tcp_stream = listener.accept().await?.0;
      let _jh = tokio::spawn(async move {
        let fun = async move {
          let net = conn_net_cb(conn_acceptor, tcp_stream).await?;
          let mut path = String::new();
          let ws = WebSocketAcceptor::default()
            .compression(conn_compression_cb())
            .req(|req| {
              if let Some(elem) = req.path {
                path.push_str(elem);
              }
              crate::Result::Ok(true)
            })
            .accept(net)
            .await?;
          conn_handle_cb.call((path, ws)).await?;
          Ok::<_, E>(())
        };
        if let Err(err) = fun.await {
          conn_conn_err(err);
        }
      });
    }
  }
}
