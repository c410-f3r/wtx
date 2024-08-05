use crate::{
  http::LowLevelServer,
  misc::{FnFut, Stream, Vector, _number_or_available_parallelism},
  pool::{Pool, SimplePoolGetElem, SimplePoolResource, SimplePoolTokio, WebSocketRM},
  rng::StdRng,
  web_socket::{Compression, FrameBuffer, FrameBufferVec, WebSocketBuffer, WebSocketServer},
};
use core::fmt::Debug;
use std::sync::OnceLock;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::MutexGuard,
};

impl LowLevelServer {
  /// Optioned WebSocket server using tokio.
  #[inline]
  pub async fn tokio_web_socket<A, C, E, F, S, SF>(
    addr: &str,
    buffers_len_opt: Option<usize>,
    compression: impl Copy + Fn() -> C + Send + 'static,
    conn_err: impl Copy + Fn(E) + Send + 'static,
    handle: F,
    (acceptor_cb, local_acceptor_cb, stream_cb): (
      impl FnOnce() -> A + Send + 'static,
      impl Copy + Fn(&A) -> A + Send + 'static,
      impl Copy + Fn(A, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    A: Send + 'static,
    C: Compression<false> + Send + 'static,
    C::NegotiatedCompression: Send,
    E: Debug + From<crate::Error> + Send + 'static,
    for<'fb, 'wsb> F: Copy
      + FnFut<
        (
          &'fb mut FrameBufferVec,
          WebSocketServer<C::NegotiatedCompression, StdRng, S, &'wsb mut WebSocketBuffer>,
        ),
        Result<(), E>,
      > + Send
      + 'static,
    S: Stream<read(..): Send, write_all(..): Send> + Send,
    SF: Send + Future<Output = crate::Result<S>>,
    for<'fb, 'wsb> <F as FnFut<
      (
        &'fb mut FrameBufferVec,
        WebSocketServer<C::NegotiatedCompression, StdRng, S, &'wsb mut WebSocketBuffer>,
      ),
      Result<(), E>,
    >>::Future: Send,
    for<'handle> &'handle F: Send,
  {
    let buffers_len = _number_or_available_parallelism(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb();
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);

      let mut conn_buffer_guard = conn_buffer(buffers_len).await?;
      let _jh = tokio::spawn(async move {
        let (fb, wsb) = &mut ***conn_buffer_guard;
        let fun = || async move {
          let stream = stream_cb(local_acceptor, tcp_stream).await?;
          handle((
            fb,
            WebSocketServer::accept(compression(), StdRng::default(), stream, wsb, |_| true)
              .await?,
          ))
          .await?;
          Ok::<_, E>(())
        };
        if let Err(err) = fun().await {
          conn_err(err);
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
    .get(&(), &())
    .await
}
