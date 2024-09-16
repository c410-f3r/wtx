use crate::{
  http::{low_level_server::LowLevelServer, ReqResBuffer, Request, Response},
  http2::{Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Either, FnFut, StreamReader, StreamWriter},
};
use core::future::Future;
use tokio::net::{TcpListener, TcpStream};

type Http2Buffer = crate::http2::Http2Buffer<ReqResBuffer>;

impl LowLevelServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_http2<ACPT, CA, E, F, RA, SR, SF, SW>(
    addr: &str,
    conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    handle_cb: F,
    req_cb: impl Clone + Fn() -> crate::Result<(RA, ReqResBuffer)> + Send + 'static,
    (acceptor_cb, local_acceptor_cb, stream_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    CA: Clone + Send + 'static,
    ACPT: Send + 'static,
    E: From<crate::Error> + Send + 'static,
    F: Clone
      + FnFut<(CA, RA, Request<ReqResBuffer>), Result = Result<Response<ReqResBuffer>, E>>
      + Send
      + 'static,
    F::Future: Send,
    RA: Send + 'static,
    SF: Send + Future<Output = crate::Result<(SR, SW)>>,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
    for<'handle> &'handle F: Send,
  {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);
      let local_conn_cb = conn_cb.clone();
      let local_err_cb = err_cb.clone();
      let local_handle_cb = handle_cb.clone();
      let local_req_cb = req_cb.clone();
      let local_stream_cb = stream_cb.clone();
      let _conn_jh = tokio::spawn(async move {
        let local_local_err_cb = local_err_cb.clone();
        let fut = manage_conn(
          local_acceptor,
          tcp_stream,
          local_conn_cb,
          local_err_cb,
          local_handle_cb,
          local_req_cb,
          local_stream_cb,
        );
        if let Err(err) = fut.await {
          local_local_err_cb(E::from(err));
        }
      });
    }
  }
}

async fn manage_conn<ACPT, CA, E, F, RA, SR, SF, SW>(
  local_acceptor: ACPT,
  tcp_stream: TcpStream,
  conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
  err_cb: impl Clone + Fn(E) + Send + 'static,
  handle_cb: F,
  req_cb: impl Clone + Fn() -> crate::Result<(RA, ReqResBuffer)> + Send + 'static,
  stream_cb: impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
) -> crate::Result<()>
where
  CA: Clone + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  F: Clone
    + FnFut<(CA, RA, Request<ReqResBuffer>), Result = Result<Response<ReqResBuffer>, E>>
    + Send
    + 'static,
  F::Future: Send,
  RA: Send + 'static,
  SF: Send + Future<Output = crate::Result<(SR, SW)>>,
  SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
  SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
  for<'handle> &'handle F: Send,
{
  let (ca, http2_buffer, http2_params) = conn_cb()?;
  let tuple = stream_cb(local_acceptor, tcp_stream).await?;
  let (frame_reader, mut http2) = Http2Tokio::accept(http2_buffer, http2_params, tuple).await?;
  let _jh = tokio::spawn(frame_reader);
  loop {
    let (ra, rrb) = req_cb()?;
    let mut http2_stream = match http2.stream(rrb).await? {
      Either::Left(_) => return Ok(()),
      Either::Right(elem) => elem,
    };
    let local_ca = ca.clone();
    let local_handle_cb = handle_cb.clone();
    let local_err_cb = err_cb.clone();
    let _stream_jh = tokio::spawn(async move {
      let fun = || async {
        let (local_rrb, opt) = http2_stream.recv_req().await?;
        let method = match opt {
          None => return Ok(()),
          Some(elem) => elem,
        };
        let req = local_rrb.into_http2_request(method);
        let res = local_handle_cb.call((local_ca, ra, req)).await?;
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
