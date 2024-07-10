use crate::{
  http::{server::OptionedServer, ReqResBuffer, Request, Response},
  http2::{Http2Params, Http2Tokio},
  misc::{FnFut, Stream},
};
use core::{fmt::Debug, future::Future, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};

type Http2Buffer = crate::http2::Http2Buffer<ReqResBuffer>;

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_http2<A, E, F, S, SF>(
    addr: SocketAddr,
    err_cb: impl Copy + Fn(E) + Send + 'static,
    handle_cb: F,
    http2_buffer_cb: fn() -> crate::Result<Http2Buffer>,
    http2_params_cb: impl Copy + Fn() -> Http2Params + Send + 'static,
    stream_buffer_cb: fn() -> crate::Result<ReqResBuffer>,
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
      + for<'any> FnFut<Request<&'any mut ReqResBuffer>, Result<Response<&'any mut ReqResBuffer>, E>>
      + Send
      + 'static,
    S: Stream + 'static,
    SF: Send + Future<Output = crate::Result<S>>,
    for<'any> &'any F: Send,
  {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb();
    loop {
      let http2_buffer = http2_buffer_cb()?;
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);
      let _conn_jh = tokio::spawn(async move {
        let fut = manage_conn(
          http2_buffer,
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

async fn manage_conn<A, E, F, S, SF>(
  http2_buffer: Http2Buffer,
  local_acceptor: A,
  tcp_stream: TcpStream,
  err_cb: impl Copy + Fn(E) + Send + 'static,
  handle_cb: F,
  http2_params_cb: impl Copy + Fn() -> Http2Params + Send + 'static,
  stream_buffer_cb: fn() -> crate::Result<ReqResBuffer>,
  stream_cb: impl Copy + Fn(A, TcpStream) -> SF + Send + 'static,
) -> crate::Result<()>
where
  E: Debug + From<crate::Error> + Send + 'static,
  F: Copy
    + for<'any> FnFut<Request<&'any mut ReqResBuffer>, Result<Response<&'any mut ReqResBuffer>, E>>
    + Send
    + 'static,
  S: Stream + 'static,
  SF: Send + Future<Output = crate::Result<S>>,
  for<'any> &'any F: Send,
{
  let stream = stream_cb(local_acceptor, tcp_stream).await?;
  let mut http2 = Http2Tokio::accept(http2_buffer, http2_params_cb(), stream).await?;
  loop {
    let rslt = http2.stream(stream_buffer_cb()?).await;
    let mut http2_stream = match rslt {
      Err(err) => match &err {
        // Closing a connection without errors
        crate::Error::Http2ErrorGoAway(_, None) => {
          drop(http2);
          return Ok(());
        }
        // Closing a connection with unexpected errors
        crate::Error::Http2ErrorGoAway(_, Some(_)) => {
          drop(http2);
          err_cb(E::from(err));
          return Ok(());
        }
        // Resetting a stream without errors
        crate::Error::Http2ErrorReset(_, None, _) => {
          continue;
        }
        // Resetting a stream with unexpected errors
        crate::Error::Http2ErrorReset(_, Some(_), _) => {
          err_cb(E::from(err));
          continue;
        }
        _ => {
          drop(http2);
          return Err(err);
        }
      },
      Ok(elem) => elem,
    };
    let _stream_jh = tokio::spawn(async move {
      let fun = || async move {
        let (mut rrb, method) = http2_stream.recv_req().await?;
        let req = rrb.as_http2_request_mut(method);
        let res = handle_cb(req).await?;
        http2_stream.send_res(res).await?;
        Ok::<_, E>(())
      };
      if let Err(err) = fun().await {
        err_cb(err);
      }
    });
  }
}
