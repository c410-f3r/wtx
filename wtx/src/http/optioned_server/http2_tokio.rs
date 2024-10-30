use crate::{
  http::{
    optioned_server::OptionedServer, AutoStream, Headers, ManualServerStreamTokio, Method,
    Protocol, ReqResBuffer, Response, StreamMode,
  },
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Either, FnFut, StreamReader, StreamWriter},
};
use core::{future::Future, mem};
use tokio::net::{TcpListener, TcpStream};

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn http2_tokio<A, ACPT, CA, E, M, N, SA, SMA, SR, SW>(
    addr: &str,
    auto_cb: A,
    conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    manual_cb: M,
    stream_cb: impl Clone + Fn() -> crate::Result<(SA, ReqResBuffer)> + Send + 'static,
    stream_mode_cb: impl Clone
      + Fn(&mut Headers, Method, Option<Protocol>) -> Result<StreamMode<SMA>, E>
      + Send
      + Sync
      + 'static,
    (acceptor_cb, conn_acceptor_cb, net_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> N + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    A: Clone
      + FnFut<(AutoStream<CA, SA>,), Result = Result<Response<ReqResBuffer>, E>>
      + Send
      + 'static,
    A::Future: Send,
    CA: Clone + Send + 'static,
    ACPT: Send + 'static,
    E: From<crate::Error> + Send + 'static,
    M: Clone
      + FnFut<(ManualServerStreamTokio<CA, Http2Buffer, SA, SMA, SW>,), Result = Result<(), E>>
      + Send
      + 'static,
    M::Future: Send,
    N: Future<Output = crate::Result<(SR, SW)>> + Send,
    SA: Send + 'static,
    SMA: Send + 'static,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
    for<'handle> &'handle A: Send,
    for<'handle> &'handle M: Send,
  {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = acceptor_cb()?;
    loop {
      let tcp_stream = listener.accept().await?.0;
      let peer = tcp_stream.peer_addr()?.ip();
      let conn_acceptor = conn_acceptor_cb(&acceptor);
      let conn_auto_cb = auto_cb.clone();
      let conn_conn_cb = conn_cb.clone();
      let conn_err_cb = err_cb.clone();
      let conn_manual_cb = manual_cb.clone();
      let conn_net_cb = net_cb.clone();
      let conn_stream_cb = stream_cb.clone();
      let conn_stream_mode_cb = stream_mode_cb.clone();
      let _conn_jh = tokio::spawn(async move {
        let another_conn_err_cb = conn_err_cb.clone();
        let conn_fun = async move {
          let (conn_ca, http2_buffer, http2_params) = conn_conn_cb()?;
          let (frame_reader, mut http2) = Http2Tokio::accept(
            http2_buffer,
            http2_params,
            conn_net_cb(conn_acceptor, tcp_stream).await?,
          )
          .await?;
          let _frame_reader_jh = tokio::spawn(frame_reader);
          loop {
            let (stream_aux, rrb) = conn_stream_cb()?;
            let (mut stream, headers_opt) = match http2
              .stream(rrb, |headers, method, protocol| {
                Ok::<_, E>(match conn_stream_mode_cb(headers, method, protocol)? {
                  StreamMode::Auto => None,
                  StreamMode::Manual(ma) => Some((mem::take(headers), ma)),
                })
              })
              .await?
            {
              Either::Left(_) => return Ok(()),
              Either::Right(elem) => elem,
            };
            let stream_auto_cb = conn_auto_cb.clone();
            let stream_ca = conn_ca.clone();
            let stream_err_cb = conn_err_cb.clone();
            let stream_manual_cb = conn_manual_cb.clone();
            let _stream_jh = tokio::spawn(async move {
              let stream_fun = async {
                if let Some((headers, stream_mode_aux)) = headers_opt? {
                  stream_manual_cb
                    .call((ManualServerStreamTokio {
                      conn_aux: stream_ca,
                      headers,
                      peer,
                      stream: stream.clone(),
                      stream_aux,
                      stream_mode_aux,
                    },))
                    .await?;
                  return Ok(());
                }
                let (hrs, local_rrb) = stream.recv_req().await?;
                if hrs.is_closed() {
                  return Ok(());
                }
                let req = local_rrb.into_http2_request(stream.method());
                let _as = AutoStream { conn_aux: stream_ca, peer, req, stream_aux };
                let res = stream_auto_cb.call((_as,)).await?;
                if stream.send_res(res).await?.is_closed() {
                  return Ok(());
                }
                Ok::<_, E>(())
              };
              let stream_fun_rslt = stream_fun.await;
              let _rslt = stream.common().clear(true).await;
              if let Err(err) = stream_fun_rslt {
                stream.common().send_go_away(Http2ErrorCode::InternalError).await;
                stream_err_cb(err);
              }
            });
          }
        };
        if let Err(err) = conn_fun.await {
          another_conn_err_cb(E::from(err));
        }
      });
    }
  }
}
