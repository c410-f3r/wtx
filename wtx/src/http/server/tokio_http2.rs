use crate::{
  http::{
    server::{TokioHttp2, _buffers_len},
    Headers, RequestStr, Response,
  },
  http2::{Http2Params, Http2Rslt, Http2Tokio},
  misc::{ByteVector, FnFut},
  pool::{
    FixedPoolGetRsltTokio, FixedPoolTokio, Http2ServerBufferRM, Pool, ReqResBufferRM,
    ResourceManager,
  },
  rng::StdRng,
};
use core::{fmt::Debug, net::SocketAddr};
use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

type ConnPool = FixedPoolTokio<
  <Http2ServerBufferRM<StdRng> as ResourceManager>::Resource,
  Http2ServerBufferRM<StdRng>,
>;
type ReqPool = FixedPoolTokio<<ReqResBufferRM as ResourceManager>::Resource, ReqResBufferRM>;

impl TokioHttp2 {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_http2<E, F>(
    addr: SocketAddr,
    buffers_len_opt: Option<usize>,
    conn_err: fn(crate::Error),
    stream_err: fn(E),
    handle: F,
  ) -> crate::Result<()>
  where
    E: Debug + From<crate::Error> + Send + 'static,
    F: Copy
      + for<'any> FnFut<
        RequestStr<'any, (&'any mut ByteVector, &'any mut Headers)>,
        Result<Response<(&'any mut ByteVector, &'any mut Headers)>, E>,
      > + Send
      + 'static,
    for<'any> &'any F: Send,
  {
    let buffers_len = _buffers_len(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let http2_lock = conn_buffer(buffers_len).await?;
      let _conn_jh = tokio::spawn(async move {
        if let Err(err) = manage_conn(handle, http2_lock, buffers_len, stream_err, tcp_stream).await
        {
          conn_err(err);
        }
      });
    }
  }
}

async fn conn_buffer(len: usize) -> crate::Result<<ConnPool as Pool>::GetRslt<'static>> {
  static POOL: OnceLock<ConnPool> = OnceLock::new();
  POOL
    .get_or_init(|| {
      FixedPoolTokio::new(len, Http2ServerBufferRM::<StdRng>::http2_buffer(StdRng::default()))
    })
    .get(&(), &())
    .await
}

async fn manage_conn<E, F>(
  handle: F,
  http2_lock: FixedPoolGetRsltTokio<'static, <ConnPool as Pool>::GuardElement>,
  len: usize,
  stream_err: fn(E),
  tcp_stream: TcpStream,
) -> crate::Result<()>
where
  E: Debug + From<crate::Error> + Send + 'static,
  F: Copy
    + for<'any> FnFut<
      RequestStr<'any, (&'any mut ByteVector, &'any mut Headers)>,
      Result<Response<(&'any mut ByteVector, &'any mut Headers)>, E>,
    > + Send
    + 'static,
  for<'any> &'any F: Send,
{
  let hp = Http2Params::default();
  let mut http2 = Http2Tokio::accept(http2_lock, hp, tcp_stream).await?;
  loop {
    let mut req_buffer_guard = req_buffer(len).await?;
    let mut stream = match http2.stream(&mut req_buffer_guard).await {
      Err(err) => {
        drop(req_buffer_guard.release().await);
        return Err(err);
      }
      Ok(Http2Rslt::ClosedConnection) => {
        drop(req_buffer_guard.release().await);
        return Ok(());
      }
      Ok(Http2Rslt::ClosedStream) => {
        drop(req_buffer_guard.release().await);
        continue;
      }
      Ok(Http2Rslt::Resource(elem)) => elem,
    };
    let _stream_jh = tokio::spawn(async move {
      let rrb = &mut **req_buffer_guard;
      let fun = || async move {
        let Http2Rslt::Resource(req) = stream.recv_req(rrb).await? else {
          return Ok(());
        };
        if stream.send_res(handle(req).await?).await?.resource().is_none() {
          return Ok(());
        }
        Ok::<_, E>(())
      };
      let rslt = fun().await;
      drop(req_buffer_guard.release().await);
      if let Err(err) = rslt {
        stream_err(err);
      }
    });
  }
}

async fn req_buffer(len: usize) -> crate::Result<<ReqPool as Pool>::GetRslt<'static>> {
  static POOL: OnceLock<ReqPool> = OnceLock::new();
  POOL
    .get_or_init(|| FixedPoolTokio::new(len, ReqResBufferRM::req_res_buffer()))
    .get(&(), &())
    .await
}
