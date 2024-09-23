use crate::{
  http::LowLevelServer,
  misc::{FnFut, Stream, Vector, Xorshift64, _number_or_available_parallelism, simple_seed},
  pool::{SimplePoolGetElem, SimplePoolResource, SimplePoolTokio, WebSocketRM},
  web_socket::{Compression, FrameBuffer, FrameBufferVec, WebSocketBuffer, WebSocketServer},
};
use core::{fmt::Debug, future::Future};
use std::sync::OnceLock;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::MutexGuard,
};

impl LowLevelServer {
  /// Optioned WebSocket server using tokio.
  #[inline]
  pub async fn tokio_web_socket<ACPT, C, E, F, S, SF>(
    addr: &str,
    buffers_len_opt: Option<usize>,
    compression_cb: impl Clone + Fn() -> C + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    handle_cb: F,
    (acceptor_cb, local_acceptor_cb, stream_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    ACPT: Send + 'static,
    C: Compression<false> + Send + 'static,
    C::NegotiatedCompression: Send,
    E: Debug + From<crate::Error> + Send + 'static,
    for<'fb, 'wsb> F: Clone
      + FnFut<
        (
          &'fb mut FrameBufferVec,
          WebSocketServer<C::NegotiatedCompression, Xorshift64, S, &'wsb mut WebSocketBuffer>,
        ),
        Result = Result<(), E>,
      > + Send
      + 'static,
    S: Stream<read(..): Send, write_all(..): Send> + Send,
    SF: Send + Future<Output = crate::Result<S>>,
    for<'fb, 'wsb> <F as FnFut<(
      &'fb mut FrameBufferVec,
      WebSocketServer<C::NegotiatedCompression, Xorshift64, S, &'wsb mut WebSocketBuffer>,
    )>>::Future: Send,
    for<'handle> &'handle F: Send,
  {
    let buffers_len = _number_or_available_parallelism(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);

      let mut conn_buffer_guard = conn_buffer(buffers_len).await?;
      let local_compression_cb = compression_cb.clone();
      let local_conn_err = err_cb.clone();
      let local_handle_cb = handle_cb.clone();
      let local_stream_cb = stream_cb.clone();
      let _jh = tokio::spawn(async move {
        let (fb, wsb) = &mut ***conn_buffer_guard;
        let fun = || async move {
          let stream = local_stream_cb(local_acceptor, tcp_stream).await?;
          local_handle_cb
            .call((
              fb,
              WebSocketServer::accept(
                local_compression_cb(),
                Xorshift64::from(simple_seed()),
                stream,
                wsb,
                |_| true,
              )
              .await?,
            ))
            .await?;
          Ok::<_, E>(())
        };
        if let Err(err) = fun().await {
          local_conn_err(err);
        }
      });
    }
  }
}

async fn conn_buffer(
  len: usize,
) -> crate::Result<
  SimplePoolGetElem<
    MutexGuard<'static, SimplePoolResource<(FrameBuffer<Vector<u8>>, WebSocketBuffer)>>,
  >,
> {
  static POOL: OnceLock<SimplePoolTokio<WebSocketRM>> = OnceLock::new();
  POOL
    .get_or_init(|| SimplePoolTokio::new(len, WebSocketRM::new(|| Ok(Default::default()))))
    .get()
    .await
}
