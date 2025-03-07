use crate::{
  http::{
    AutoStream, ManualServerStreamTokio, OperationMode, Protocol, ReqResBuffer, Request, Response,
    optioned_server::OptionedServer,
  },
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Either, FnFut, StreamReader, StreamWriter},
};
use core::mem;
use tokio::net::{TcpListener, TcpStream};

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  #[inline]
  pub async fn http2_tokio<A, ACPT, CA, E, HA, M, N, OM, SA, SR, SW>(
    addr: &str,
    auto_cb: A,
    conn_cb: impl Clone + Fn() -> crate::Result<(CA, Http2Buffer, Http2Params)> + Send + 'static,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    manual_cb: M,
    operation_mode: OM,
    stream_cb: impl Clone + Fn() -> crate::Result<(SA, ReqResBuffer)> + Send + 'static,
    (acceptor_cb, conn_acceptor_cb, net_cb): (
      impl FnOnce() -> crate::Result<ACPT> + Send + 'static,
      impl Clone + Fn(&ACPT) -> ACPT + Send + 'static,
      impl Clone + Fn(ACPT, TcpStream) -> N + Send + 'static,
    ),
  ) -> crate::Result<()>
  where
    A: Clone
      + FnFut<(HA, AutoStream<CA, SA>), Result = Result<Response<ReqResBuffer>, E>>
      + Send
      + 'static,
    A::Future: Send,
    ACPT: Send + 'static,
    CA: Clone + Send + 'static,
    E: From<crate::Error> + Send + 'static,
    HA: Send + 'static,
    M: Clone
      + FnFut<(HA, ManualServerStreamTokio<CA, Http2Buffer, SA, SW>), Result = Result<(), E>>
      + Send
      + 'static,
    M::Future: Send,
    N: Future<Output = crate::Result<(SR, SW)>> + Send,
    OM: Clone
      + Fn(&CA, Option<Protocol>, Request<&mut ReqResBuffer>, &SA) -> Result<(HA, OperationMode), E>
      + Send
      + 'static,
    SA: Send + 'static,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + Unpin + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + Unpin + 'static,
    for<'any> &'any A: Send,
    for<'any> &'any CA: Send,
    for<'any> &'any M: Send,
    for<'any> &'any OM: Send,
    for<'any> &'any SA: Send,
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
      let conn_op_cb = operation_mode.clone();
      let _conn_jh = tokio::spawn(async move {
        let initial = async move {
          let (conn_ca, http2_buffer, http2_params) = conn_conn_cb()?;
          let (frame_reader, http2) = Http2Tokio::accept(
            http2_buffer,
            http2_params,
            conn_net_cb(conn_acceptor, tcp_stream).await?,
          )
          .await?;
          Ok::<_, E>((conn_ca, frame_reader, http2))
        };
        let (conn_ca, frame_reader, mut http2) = match initial.await {
          Err(err) => {
            conn_err_cb(err);
            return;
          }
          Ok(elem) => elem,
        };
        let another_conn_err_cb = conn_err_cb.clone();
        let another_http2 = http2.clone();
        let _frame_reader_jh = tokio::spawn(frame_reader);
        let rest = async move {
          loop {
            let stream_ca = conn_ca.clone();
            let (stream_aux, rrb) = conn_stream_cb()?;
            let (mut stream, rslt) = match http2
              .stream(rrb, |req, protocol| {
                let op = conn_op_cb(
                  &stream_ca,
                  protocol,
                  Request { method: req.method, rrd: &mut *req.rrd, version: req.version },
                  &stream_aux,
                )?;
                Ok::<_, E>(match op.1 {
                  OperationMode::Auto => (op.0, None),
                  OperationMode::Manual => (op.0, Some(mem::take(req.rrd))),
                })
              })
              .await?
            {
              Either::Left(_) => return Ok(()),
              Either::Right(elem) => elem,
            };
            let (headers_aux, opt) = rslt?;
            let stream_auto_cb = conn_auto_cb.clone();
            let stream_err_cb = conn_err_cb.clone();
            let stream_manual_cb = conn_manual_cb.clone();
            let _stream_jh = tokio::spawn(async move {
              let stream_fun = async {
                if let Some(local_rrb) = opt {
                  stream_manual_cb
                    .call((
                      headers_aux,
                      ManualServerStreamTokio {
                        conn_aux: stream_ca,
                        peer,
                        protocol: stream.protocol(),
                        req: Request::http2(stream.method(), local_rrb),
                        stream: stream.clone(),
                        stream_aux,
                      },
                    ))
                    .await?;
                  return Ok(());
                }
                let (hrs, local_rrb) = stream.recv_req().await?;
                if hrs.is_closed() {
                  return Ok(());
                }
                let req = local_rrb.into_http2_request(stream.method());
                let auto_stream = AutoStream {
                  conn_aux: stream_ca,
                  peer,
                  protocol: stream.protocol(),
                  req,
                  stream_aux,
                };
                let res = stream_auto_cb.call((headers_aux, auto_stream)).await?;
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
        if let Err(err) = rest.await {
          another_http2.send_go_away(Http2ErrorCode::NoError).await;
          another_conn_err_cb(err);
        }
      });
    }
  }
}
