use crate::{
  http::OptionedServer,
  misc::FnFut,
  pool::{SimplePoolTokio, WebSocketRM},
  rng::Xorshift64,
  stream::Stream,
  web_socket::{Compression, WebSocket, WebSocketAcceptor, WebSocketBuffer},
};
use core::fmt::Debug;
use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

static POOL: OnceLock<SimplePoolTokio<WebSocketRM>> = OnceLock::new();

impl OptionedServer {
  /// Optioned WebSocket server using tokio.
  #[inline]
  pub async fn web_socket_tokio<ACPT, C, E, H, N, S>(
    addr: &str,
    buffers_len_opt: Option<usize>,
    compression_cb: impl Clone + Fn() -> C + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    handle_cb: H,
    (acceptor_cb, conn_acceptor_cb, net_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> N + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    ACPT: Send + 'static,
    C: Compression<false> + Send + 'static,
    C::NegotiatedCompression: Send,
    E: Debug + From<crate::Error> + Send + 'static,
    for<'wsb> H: Clone
      + FnFut<
        (WebSocket<C::NegotiatedCompression, Xorshift64, S, &'wsb mut WebSocketBuffer, false>,),
        Result = Result<(), E>,
      > + Send
      + 'static,
    N: Send + Future<Output = crate::Result<S>>,
    S: Stream<read(..): Send, write_all(..): Send> + Send,
    for<'wsb> <H as FnFut<(
      WebSocket<C::NegotiatedCompression, Xorshift64, S, &'wsb mut WebSocketBuffer, false>,
    )>>::Future: Send,
    for<'handle> &'handle H: Send,
  {
    let buffers_len = number_or_available_parallelism(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let conn_acceptor = conn_acceptor_cb(&acceptor);
      let conn_compression_cb = compression_cb.clone();
      let conn_conn_err = err_cb.clone();
      let conn_handle_cb = handle_cb.clone();
      let conn_net_cb = net_cb.clone();
      let tcp_stream = listener.accept().await?.0;
      let mut conn_buffer = POOL
        .get_or_init(|| {
          SimplePoolTokio::new(buffers_len, WebSocketRM::new(|| Ok(Default::default())))
        })
        .get()
        .await?;
      let _jh = tokio::spawn(async move {
        let wsb = &mut ***conn_buffer;
        let fun = async move {
          let net = conn_net_cb(conn_acceptor, tcp_stream).await?;
          conn_handle_cb
            .call((WebSocketAcceptor::default()
              .compression(conn_compression_cb())
              .no_masking(true)
              .wsb(wsb)
              .accept(net)
              .await?,))
            .await?;
          Ok::<_, E>(())
        };
        if let Err(err) = fun.await {
          conn_conn_err(err);
        }
      });
    }
  }
}

pub(crate) fn number_or_available_parallelism(n: Option<usize>) -> crate::Result<usize> {
  Ok(if let Some(elem) = n { elem } else { usize::from(std::thread::available_parallelism()?) })
}
