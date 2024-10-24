use crate::{
  http::{optioned_server::OptionedServer, Headers, ReqResBuffer, Request, Response},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio, ServerStreamTokio},
  misc::{Either, FnFut, StreamReader, StreamWriter},
};
use core::future::Future;
use tokio::net::{TcpListener, TcpStream};

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn tokio_high_http2<A, ACPT, CA, E, M, SA, SR, SF, SW>(
    addr: &str,
    auto_cb: A,
    conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    manual_cb: M,
    stream_cb: impl Clone + Fn() -> crate::Result<(SA, ReqResBuffer)> + Send + 'static,
    (acceptor_cb, local_acceptor_cb, net_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    A: Clone
      + FnFut<(CA, SA, Request<ReqResBuffer>), Result = Result<Response<ReqResBuffer>, E>>
      + Send
      + 'static,
    A::Future: Send,
    CA: Clone + Send + 'static,
    ACPT: Send + 'static,
    E: From<crate::Error> + Send + 'static,
    M: Clone
      + FnFut<(CA, SA, Headers, ServerStreamTokio<Http2Buffer, SW, false>), Result = Result<(), E>>
      + Send
      + 'static,
    M::Future: Send,
    SA: Send + 'static,
    SF: Send + Future<Output = crate::Result<(SR, SW)>>,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
    for<'handle> &'handle A: Send,
    for<'handle> &'handle M: Send,
  {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let (tcp_stream, _) = listener.accept().await?;
      let local_acceptor = local_acceptor_cb(&acceptor);
      let local_auto_cb = auto_cb.clone();
      let local_conn_cb = conn_cb.clone();
      let local_err_cb = err_cb.clone();
      let local_manual_cb = manual_cb.clone();
      let local_net_cb = net_cb.clone();
      let local_stream_cb = stream_cb.clone();
      let _conn_jh = tokio::spawn(async move {
        let local_local_err_cb = local_err_cb.clone();
        let fut = manage_conn(
          local_acceptor,
          local_auto_cb,
          local_conn_cb,
          local_err_cb,
          local_manual_cb,
          local_net_cb,
          local_stream_cb,
          tcp_stream,
        );
        if let Err(err) = fut.await {
          local_local_err_cb(E::from(err));
        }
      });
    }
  }
}

async fn manage_conn<A, ACPT, CA, E, M, SA, SR, SF, SW>(
  acceptor: ACPT,
  auto_cb: A,
  conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
  err_cb: impl Clone + Fn(E) + Send + 'static,
  manual_cb: M,
  net_cb: impl Clone + Fn(ACPT, TcpStream) -> SF + Send + 'static,
  stream_cb: impl Clone + Fn() -> crate::Result<(SA, ReqResBuffer)> + Send + 'static,
  tcp_stream: TcpStream,
) -> crate::Result<()>
where
  A: Clone
    + FnFut<(CA, SA, Request<ReqResBuffer>), Result = Result<Response<ReqResBuffer>, E>>
    + Send
    + 'static,
  A::Future: Send,
  CA: Clone + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  M: Clone
    + FnFut<(CA, SA, Headers, ServerStreamTokio<Http2Buffer, SW, false>), Result = Result<(), E>>
    + Send
    + 'static,
  M::Future: Send,
  SA: Send + 'static,
  SF: Send + Future<Output = crate::Result<(SR, SW)>>,
  SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
  SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
  for<'handle> &'handle A: Send,
  for<'handle> &'handle M: Send,
{
  let (ca, http2_buffer, http2_params) = conn_cb()?;
  let net_tuple = net_cb(acceptor, tcp_stream).await?;
  let accept_tuple = Http2Tokio::accept(http2_buffer, http2_params, net_tuple).await?;
  let (frame_reader, mut http2) = accept_tuple;
  let _jh = tokio::spawn(frame_reader);
  loop {
    let (ra, rrb) = stream_cb()?;
    let (mut http2_stream, _headers_opt) = match http2
      .stream(rrb, |_headers, _method, _protocol| {
        #[cfg(feature = "web-socket")]
        {
          let is_ws = crate::http2::is_web_socket_handshake(_headers, _method, _protocol);
          is_ws.then(|| core::mem::take(_headers))
        }
      })
      .await?
    {
      Either::Left(_) => return Ok(()),
      Either::Right(elem) => elem,
    };
    let local_auto_cb = auto_cb.clone();
    let local_ca = ca.clone();
    let local_err_cb = err_cb.clone();
    let _local_manual_cb = manual_cb.clone();
    let _stream_jh = tokio::spawn(async move {
      let fun = async {
        #[cfg(feature = "web-socket")]
        if let Some(headers) = _headers_opt {
          _local_manual_cb.call((local_ca, ra, headers, http2_stream.clone())).await?;
          return Ok(());
        }
        let (hrs, local_rrb) = http2_stream.recv_req().await?;
        if hrs.is_closed() {
          return Ok(());
        }
        let req = local_rrb.into_http2_request(http2_stream.method());
        let res = local_auto_cb.call((local_ca, ra, req)).await?;
        if http2_stream.send_res(res).await?.is_closed() {
          return Ok(());
        }
        Ok::<_, E>(())
      };
      let rslt = fun.await;
      let _rslt = http2_stream.common().clear(true).await;
      if let Err(err) = rslt {
        http2_stream.common().send_go_away(Http2ErrorCode::InternalError).await;
        local_err_cb(err);
      }
    });
  }
}
