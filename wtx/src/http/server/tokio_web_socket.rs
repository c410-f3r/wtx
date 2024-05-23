use crate::{
  http::server::{TokioWebSocket, _buffers_len},
  misc::FnFut,
  pool::{Pool, ResourceManager, SimplePoolTokio, WebSocketRM},
  rng::StdRng,
  web_socket::{
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    Compression, FrameBufferVec, WebSocketBuffer, WebSocketServer,
  },
};
use core::{fmt::Debug, net::SocketAddr};
use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

type WebSocketPool = SimplePoolTokio<<WebSocketRM as ResourceManager>::Resource, WebSocketRM>;

impl TokioWebSocket {
  /// Optioned WebSocket server using tokio.
  #[inline]
  pub async fn tokio_web_socket<C, E, F>(
    addr: SocketAddr,
    buffers_len_opt: Option<usize>,
    compression: fn() -> C,
    conn_err: fn(E),
    handle: F,
  ) -> crate::Result<()>
  where
    C: Compression<false> + Send + 'static,
    C::NegotiatedCompression: Send,
    E: Debug + From<crate::Error> + Send + 'static,
    F: Copy
      + for<'any> FnFut<
        (
          &'any mut FrameBufferVec,
          WebSocketServer<C::NegotiatedCompression, StdRng, TcpStream, &'any mut WebSocketBuffer>,
        ),
        Result<(), E>,
      > + Send
      + 'static,
    for<'any> &'any F: Send,
  {
    let buffers_len = _buffers_len(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    loop {
      let (stream, _) = listener.accept().await?;
      let mut conn_buffer_guard = conn_buffer(buffers_len).await?;
      let _jh = tokio::spawn(async move {
        let (fb, wsb) = &mut ***conn_buffer_guard;
        let fun = || async move {
          handle((
            fb,
            WebSocketAcceptRaw { compression: compression(), rng: StdRng::default(), stream, wsb }
              .accept(|_| true)
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

async fn conn_buffer(len: usize) -> crate::Result<<WebSocketPool as Pool>::GetElem<'static>> {
  static POOL: OnceLock<WebSocketPool> = OnceLock::new();
  POOL.get_or_init(|| SimplePoolTokio::new(len, WebSocketRM::web_socket())).get(&(), &()).await
}
