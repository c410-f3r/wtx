use crate::{
  calendar::Instant,
  http::{
    AutoStream, ManualServerStreamTokio, OperationMode, Protocol, ReqResBuffer, Request, Response,
    optioned_server::OptionedServer,
  },
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Either, FnFut},
  stream::{StreamReader, StreamWriter},
};
use core::{mem, net::IpAddr};
use tokio::net::{TcpListener, TcpStream};

impl OptionedServer {
  /// Optioned HTTP/2 server using tokio.
  ///
  /// The order of the callbacks roughly represents their execution order.
  //
  // It is not possible to use a struct to wrap the callbacks because the compiler asks for
  // explicit types declarations at call-site.
  #[inline]
  pub async fn http2_tokio<
    ACPT,
    CA,
    ERR,
    HA,
    HCAC,
    HCACP,
    HCEC,
    HCOC,
    HCOCP,
    HCSC,
    HSAC,
    HSEC,
    HSMC,
    SA,
    SR,
    SW,
    TAC,
    TSC,
    TSF,
  >(
    (acpt, addr, mut hcacp, hcocp): (ACPT, &str, HCACP, HCOCP),
    tcp_acceptance_cb: TAC,
    tcp_stream: TSC,
    http2_conn_error_cb: HCEC,
    http2_conn_acceptance_cb: HCAC,
    http2_conn_stream_cb: HCSC,
    http2_conn_om_cb: HCOC,
    http2_stream_error_cb: HSEC,
    http2_stream_auto_cb: HSAC,
    http2_stream_manual_cb: HSMC,
  ) -> Result<(), ERR>
  where
    ACPT: Clone + Send + 'static,
    TAC: Fn(&mut HCACP) -> Result<(), ERR> + Send + 'static,
    TSC: Clone + Fn(ACPT, TcpStream) -> TSF + Send + 'static,
    HCEC: Clone + Fn(ERR) + Send + 'static,
    HCAC: Clone + Fn(HCACP) -> Result<(CA, Http2Buffer, Http2Params), ERR> + Send + 'static,
    HCACP: Clone + Send + 'static,
    HCSC: Clone + Fn(&mut CA) -> Result<(SA, ReqResBuffer), ERR> + Send + 'static,
    HCOC: Clone
      + Fn(
        &CA,
        &mut HCOCP,
        Option<Protocol>,
        Request<&mut ReqResBuffer>,
        &SA,
      ) -> Result<(HA, OperationMode), ERR>
      + Send
      + 'static,
    HCOCP: Clone + Send + 'static,
    HSEC: Clone + Fn(ERR) + Send + 'static,
    HSAC: Clone
      + FnFut<(HA, AutoStream<CA, SA>), Result = Result<Response<ReqResBuffer>, ERR>>
      + Send
      + 'static,
    HSMC: Clone
      + FnFut<(HA, ManualServerStreamTokio<CA, Http2Buffer, SA, SW>), Result = Result<(), ERR>>
      + Send
      + 'static,
    CA: Clone + Send + 'static,
    ERR: From<crate::Error> + Send,
    HA: Send + 'static,
    SA: Send + 'static,
    HSAC::Future: Send,
    HSMC::Future: Send,
    SR: Send + StreamReader<read(..): Send, read_skip(..): Send> + 'static,
    SW: Send + StreamWriter<write_all(..): Send, write_all_vectored(..): Send> + 'static,
    TSF: Future<Output = Result<(SR, SW), ERR>> + Send,
    for<'any> &'any CA: Send,
    for<'any> &'any HCOC: Send,
    for<'any> &'any HSAC: Send,
    for<'any> &'any HSMC: Send,
    for<'any> &'any SA: Send,
  {
    let listener = TcpListener::bind(addr).await.map_err(crate::Error::from)?;
    loop {
      let accepted_stream = listener.accept().await.map_err(crate::Error::from)?.0;
      tcp_acceptance_cb(&mut hcacp)?;

      let conn_acpt = acpt.clone();
      let conn_hcacp = hcacp.clone();
      let conn_http2_acceptance = http2_conn_acceptance_cb.clone();
      let conn_http2_error = http2_conn_error_cb.clone();
      let conn_http2_om = http2_conn_om_cb.clone();
      let conn_http2_stream = http2_conn_stream_cb.clone();
      let conn_stream_auto = http2_stream_auto_cb.clone();
      let conn_stream_error = http2_stream_error_cb.clone();
      let conn_stream_manual = http2_stream_manual_cb.clone();
      let conn_tcp_stream = tcp_stream.clone();
      let peer = accepted_stream.peer_addr().map_err(crate::Error::from)?.ip();
      let mut conn_hcocp = hcocp.clone();

      let _conn_jh = tokio::spawn(async move {
        let initial_fut = async move {
          let (ca, hb, hp) = conn_http2_acceptance(conn_hcacp)?;
          let parts = conn_tcp_stream(conn_acpt, accepted_stream).await?;
          let (frame_reader, http2) = Http2Tokio::accept(hb, hp, parts).await?;
          Ok::<_, ERR>((ca, frame_reader, http2))
        };
        let (mut conn_ca, frame_reader, mut http2) = match initial_fut.await {
          Err(err) => {
            conn_http2_error(err);
            return;
          }
          Ok(elem) => elem,
        };
        let another_http2 = http2.clone();
        let _frame_reader_jh = tokio::spawn(frame_reader);
        let rest = async move {
          loop {
            // !!! The line order is important !!!
            let (stream_aux, rrb) = conn_http2_stream(&mut conn_ca)?;
            let stream_ca = conn_ca.clone();
            // !!! The line order is important !!!
            let (mut stream, rslt) = match http2
              .stream(rrb, |req, protocol| {
                let op = conn_http2_om(
                  &stream_ca,
                  &mut conn_hcocp,
                  protocol,
                  Request { method: req.method, rrd: &mut *req.rrd, version: req.version },
                  &stream_aux,
                )?;
                Ok::<_, ERR>(match op.1 {
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
            let stream_auto_cb = conn_stream_auto.clone();
            let stream_err_cb = conn_stream_error.clone();
            let stream_manual_cb = conn_stream_manual.clone();
            let _stream_jh = tokio::spawn(async move {
              let stream_fun = async {
                if let Some(local_rrb) = opt {
                  let req = Request::http2(stream.method(), local_rrb);
                  log_req(&peer, &req);
                  stream_manual_cb
                    .call((
                      headers_aux,
                      ManualServerStreamTokio {
                        conn_aux: stream_ca,
                        peer,
                        protocol: stream.protocol(),
                        req,
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
                log_req(&peer, &req);
                let auto_stream = AutoStream {
                  conn_aux: stream_ca,
                  peer,
                  protocol: stream.protocol(),
                  req,
                  stream_aux,
                };
                let res = stream_auto_cb.call((headers_aux, auto_stream)).await?;
                let _ = stream.send_res(res).await?;
                Ok::<_, ERR>(())
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
          conn_http2_error(err);
        }
      });
    }
  }
}

fn log_req(_peer: &IpAddr, _req: &Request<ReqResBuffer>) {
  let _method = _req.method.strings().custom[0];
  let _path = _req.rrd.uri.path();
  let _version = _req.version.strings().custom[0];
  let _time = Instant::now_timestamp(0).unwrap_or_default().as_secs().cast_signed();
  let _time_display = crate::calendar::DateTime::from_timestamp_secs(_time).unwrap_or_default();
  _debug!(r#"{_peer} "{_method} {_path} {_version}""#,);
}
