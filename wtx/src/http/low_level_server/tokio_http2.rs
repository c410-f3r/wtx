use crate::{
  http::{low_level_server::LowLevelServer, ReqResBuffer, Request, Response},
  http2::{Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Either, FnFut2, StreamReader, StreamWriter},
};
use core::future::Future;
use tokio::net::{TcpListener, TcpStream};

type Http2Buffer = crate::http2::Http2Buffer<ReqResBuffer>;

impl LowLevelServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_http2<ACPT, AUX, E, F, SR, SW, SF>(
    aux: AUX,
    addr: &str,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    handle_cb: F,
    http2_buffer_cb: fn() -> crate::Result<Http2Buffer>,
    http2_params_cb: impl Clone + Fn() -> Http2Params + Send + 'static,
    stream_buffer_cb: fn() -> crate::Result<ReqResBuffer>,
    (acceptor_cb, local_acceptor_cb, stream_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    ACPT: Send + 'static,
    AUX: Clone + Send + 'static,
    E: From<crate::Error> + Send + 'static,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
    SF: Send + Future<Output = crate::Result<(SR, SW)>>,
    F: Clone
      + FnFut2<Request<ReqResBuffer>, AUX, Result = Result<Response<ReqResBuffer>, E>>
      + Send
      + 'static,
    F::Future: Send,
    for<'handle> &'handle F: Send,
  {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let http2_buffer = http2_buffer_cb()?;
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);
      let local_aux = aux.clone();
      let local_err_cb = err_cb.clone();
      let local_handle_cb = handle_cb.clone();
      let local_http2_params_cb = http2_params_cb.clone();
      let local_stream_cb = stream_cb.clone();
      let _conn_jh = tokio::spawn(async move {
        let local_local_err_cb = local_err_cb.clone();
        let fut = manage_conn(
          local_aux,
          http2_buffer,
          local_acceptor,
          tcp_stream,
          local_err_cb,
          local_handle_cb,
          local_http2_params_cb,
          stream_buffer_cb,
          local_stream_cb,
        );
        if let Err(err) = fut.await {
          local_local_err_cb(E::from(err));
        }
      });
    }
  }
}

async fn manage_conn<ACPT, AUX, E, F, SR, SW, SF>(
  aux: AUX,
  http2_buffer: Http2Buffer,
  local_acceptor: ACPT,
  tcp_stream: TcpStream,
  err_cb: impl Clone + Fn(E) + Send + 'static,
  handle_cb: F,
  http2_params_cb: impl Clone + Fn() -> Http2Params + Send + 'static,
  stream_buffer_cb: fn() -> crate::Result<ReqResBuffer>,
  stream_cb: impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
) -> crate::Result<()>
where
  AUX: Clone + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
  SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
  SF: Send + Future<Output = crate::Result<(SR, SW)>>,
  F: Clone
    + FnFut2<Request<ReqResBuffer>, AUX, Result = Result<Response<ReqResBuffer>, E>>
    + Send
    + 'static,
  F::Future: Send,
  for<'handle> &'handle F: Send,
{
  let (frame_reader, mut http2) = Http2Tokio::accept(
    http2_buffer,
    http2_params_cb(),
    stream_cb(local_acceptor, tcp_stream).await?,
  )
  .await?;
  {
    let local_err_cb = err_cb.clone();
    let _jh = tokio::spawn(async move {
      if let Err(err) = frame_reader.await {
        local_err_cb(err.into());
      }
    });
  }
  loop {
    let mut http2_stream = match http2.stream(stream_buffer_cb()?).await {
      Either::Left(_) => return Ok(()),
      Either::Right(elem) => elem,
    };
    let local_aux = aux.clone();
    let local_handle_cb = handle_cb.clone();
    let local_err_cb = err_cb.clone();
    let _stream_jh = tokio::spawn(async move {
      let fun = || async {
        let (rrb, method) = match http2_stream.recv_req().await? {
          Either::Left(_) => return Ok(()),
          Either::Right(elem) => elem,
        };
        let req = rrb.into_http2_request(method);
        let res = local_handle_cb(req, local_aux).await?;
        if http2_stream.send_res(res).await?.is_none() {
          return Ok(());
        }
        Ok::<_, E>(())
      };
      if let Err(err) = fun().await {
        http2_stream.send_go_away(Http2ErrorCode::InternalError).await;
        local_err_cb(err);
      }
    });
  }
}
