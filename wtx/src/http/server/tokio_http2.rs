use crate::{
  http::{
    server::{TokioHttp2, _buffers_len},
    Headers, RequestStr, Response,
  },
  http2::{Http2Buffer, Http2Params, Http2Tokio, StreamBuffer},
  misc::{ByteVector, FnFut},
  pool::{Http2ServerBufferRM, Pool, SimplePoolGetElemTokio, SimplePoolTokio, StreamBufferRM},
  rng::StdRng,
};
use core::{fmt::Debug, net::SocketAddr};
use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

type ConnPool = SimplePoolTokio<Http2Buffer<SB>, RM>;
type RM = Http2ServerBufferRM<StdRng, SB>;
type SB = <StreamPool as Pool>::GetElem<'static>;
type StreamPool = SimplePoolTokio<StreamBuffer, StreamBufferRM>;

impl TokioHttp2<SB> {
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

async fn conn_buffer(len: usize) -> crate::Result<<ConnPool as Pool>::GetElem<'static>> {
  static POOL: OnceLock<ConnPool> = OnceLock::new();
  POOL
    .get_or_init(|| SimplePoolTokio::new(len, RM::http2_buffer(StdRng::default())))
    .get(&(), &())
    .await
}

async fn manage_conn<E, F>(
  handle: F,
  http2_lock: SimplePoolGetElemTokio<'static, Http2Buffer<SB>>,
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
    let sb_guard = stream_buffer(len).await?;
    let mut stream = match http2.stream(sb_guard).await {
      Err(crate::Error::Http2ErrorReset(_, _)) => {
        continue;
      }
      Err(crate::Error::Http2ErrorGoAway(_, _)) => {
        return Ok(());
      }
      Err(err) => {
        return Err(err);
      }
      Ok(elem) => elem,
    };
    let _stream_jh = tokio::spawn(async move {
      let fun = || async move {
        let (mut sb, method) = stream.recv_req().await?;
        let StreamBuffer { hpack_enc_buffer, rrb } = &mut ***sb;
        let req = rrb.as_http2_request_mut(method);
        let _ = stream.send_res(hpack_enc_buffer, handle(req).await?).await?;
        Ok::<_, E>(())
      };
      if let Err(err) = fun().await {
        stream_err(err);
      }
    });
  }
}

async fn stream_buffer(len: usize) -> crate::Result<<StreamPool as Pool>::GetElem<'static>> {
  static POOL: OnceLock<StreamPool> = OnceLock::new();
  POOL
    .get_or_init(|| SimplePoolTokio::new(len, StreamBufferRM::req_res_buffer()))
    .get(&(), &())
    .await
}
