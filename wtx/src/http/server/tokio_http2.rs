use crate::{
  http::{
    server::{OptionedServer, _buffers_len},
    Headers, RequestStr, Response,
  },
  http2::{Http2Params, Http2Tokio, StreamBuffer},
  misc::{ByteVector, FnFut, Stream},
  pool::{Http2BufferRM, Pool, SimplePoolGetElemTokio, SimplePoolTokio, SimpleRM, StreamBufferRM},
};
use core::{fmt::Debug, future::Future, net::SocketAddr};
use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

type Http2Buffer = crate::http2::Http2Buffer<SimplePoolGetElemTokio<'static, StreamBuffer>>;

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_http2<A, E, F, S, SF>(
    addr: SocketAddr,
    buffers_len_opt: Option<usize>,
    err_cb: impl Copy + Fn(E) + Send + 'static,
    handle_cb: F,
    http2_buffer_cb: fn() -> crate::Result<Http2Buffer>,
    http2_params_cb: impl Copy + Fn() -> Http2Params + Send + 'static,
    stream_buffer_cb: fn() -> crate::Result<StreamBuffer>,
    (acceptor_cb, local_acceptor_cb, stream_cb): (
      impl FnOnce() -> A + Send + 'static,
      impl Copy + Fn(&A) -> A + Send + 'static,
      impl Copy + Fn(A, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    A: Send + 'static,
    E: Debug + From<crate::Error> + Send + 'static,
    F: Copy
      + for<'any> FnFut<
        RequestStr<'any, (&'any mut ByteVector, &'any mut Headers)>,
        Result<Response<(&'any mut ByteVector, &'any mut Headers)>, E>,
      > + Send
      + 'static,
    S: Stream + 'static,
    SF: Send + Future<Output = crate::Result<S>>,
    for<'any> &'any F: Send,
  {
    let buffers_len = _buffers_len(buffers_len_opt)?;
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb();
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);
      let http2_buffer = conn_buffer(buffers_len, http2_buffer_cb).await?;
      let _conn_jh = tokio::spawn(async move {
        let fut = manage_conn(
          http2_buffer,
          buffers_len,
          local_acceptor,
          tcp_stream,
          err_cb,
          handle_cb,
          http2_params_cb,
          stream_buffer_cb,
          stream_cb,
        );
        if let Err(err) = fut.await {
          err_cb(E::from(err));
        }
      });
    }
  }
}

async fn conn_buffer(
  len: usize,
  http2_buffer_cb: fn() -> crate::Result<Http2Buffer>,
) -> crate::Result<SimplePoolGetElemTokio<'static, Http2Buffer>> {
  static POOL: OnceLock<
    SimplePoolTokio<Http2BufferRM<SimplePoolGetElemTokio<'static, StreamBuffer>>>,
  > = OnceLock::new();
  POOL.get_or_init(|| SimplePoolTokio::new(len, SimpleRM::new(http2_buffer_cb))).get(&(), &()).await
}

async fn manage_conn<A, E, F, S, SF>(
  http2_buffer: SimplePoolGetElemTokio<'static, Http2Buffer>,
  len: usize,
  local_acceptor: A,
  tcp_stream: TcpStream,
  err_cb: impl Copy + Fn(E) + Send + 'static,
  handle_cb: F,
  http2_params_cb: impl Copy + Fn() -> Http2Params + Send + 'static,
  stream_buffer_cb: fn() -> crate::Result<StreamBuffer>,
  stream_cb: impl Copy + Fn(A, TcpStream) -> SF + Send + 'static,
) -> crate::Result<()>
where
  E: Debug + From<crate::Error> + Send + 'static,
  F: Copy
    + for<'any> FnFut<
      RequestStr<'any, (&'any mut ByteVector, &'any mut Headers)>,
      Result<Response<(&'any mut ByteVector, &'any mut Headers)>, E>,
    > + Send
    + 'static,
  S: Stream + 'static,
  SF: Send + Future<Output = crate::Result<S>>,
  for<'any> &'any F: Send,
{
  let stream = stream_cb(local_acceptor, tcp_stream).await?;
  let mut http2 = Http2Tokio::accept(http2_buffer, http2_params_cb(), stream).await?;
  loop {
    let sb_guard = stream_buffer(len, stream_buffer_cb).await?;
    let rslt = http2.stream(sb_guard).await;
    let mut http2_stream = match rslt {
      Err(err) => match &err {
        crate::Error::Http2ErrorGoAway(..) => {
          err_cb(E::from(err));
          return Ok(());
        }
        crate::Error::Http2ErrorReset(..) => {
          err_cb(E::from(err));
          continue;
        }
        _ => {
          return Err(err);
        }
      },
      Ok(elem) => elem,
    };
    let _stream_jh = tokio::spawn(async move {
      let fun = || async move {
        let (mut sb, method) = http2_stream.recv_req().await?;
        let StreamBuffer { hpack_enc_buffer, rrb } = &mut ***sb;
        let req = rrb.as_http2_request_mut(method);
        let res = handle_cb(req).await?;
        http2_stream.send_res(hpack_enc_buffer, res).await?;
        Ok::<_, E>(())
      };
      if let Err(err) = fun().await {
        err_cb(err);
      }
    });
  }
}

async fn stream_buffer(
  len: usize,
  stream_buffer_cb: fn() -> crate::Result<StreamBuffer>,
) -> crate::Result<SimplePoolGetElemTokio<'static, StreamBuffer>> {
  static POOL: OnceLock<SimplePoolTokio<StreamBufferRM>> = OnceLock::new();
  POOL
    .get_or_init(|| SimplePoolTokio::new(len, StreamBufferRM::new(stream_buffer_cb)))
    .get(&(), &())
    .await
}
