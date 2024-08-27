use crate::{
  http::{Method, ReqResBuffer, ReqResData, Response},
  http2::{
    misc::{
      manage_recurrent_stream_receiving, process_higher_operation_err, send_go_away,
      send_reset_stream,
    },
    send_msg::send_msg,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2ErrorCode,
    StreamControlRecvParams, U31,
  },
  misc::{Either, Lease, LeaseMut, Lock, RefCounter, StreamWriter, _Span, sleep},
};
use alloc::sync::Arc;
use core::{
  future::{poll_fn, Future},
  pin::pin,
  sync::atomic::AtomicBool,
  time::Duration,
};

/// Created when a server receives an initial stream.
#[derive(Debug)]
pub struct ServerStream<HD> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  method: Method,
  span: _Span,
  stream_id: U31,
}

impl<HD> ServerStream<HD> {
  #[inline]
  pub(crate) const fn new(
    hd: HD,
    is_conn_open: Arc<AtomicBool>,
    method: Method,
    span: _Span,
    stream_id: U31,
  ) -> Self {
    Self { hd, is_conn_open, method, span, stream_id }
  }
}

impl<HB, HD, RRB, SW> ServerStream<HD>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, false>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  /// Receive request
  ///
  /// Higher operation that awaits for the data necessary to build a request.
  ///
  /// Returns [`Either::Left`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn recv_req(&mut self) -> crate::Result<Either<RRB, (RRB, Method)>> {
    let Self { hd, is_conn_open, method, span, stream_id } = self;
    let _e = span._enter();
    _trace!("Receiving request");
    let mut lock_pin = pin!(hd.lock());
    let rslt = poll_fn(|cx| {
      let mut lock = lock_pin!(cx, hd, lock_pin);
      manage_recurrent_stream_receiving(
        cx,
        lock.parts_mut(),
        is_conn_open,
        *stream_id,
        |local_cx, hdpm, sorp| {
          drop(hdpm.hb.scrp.insert(
            *stream_id,
            StreamControlRecvParams {
              is_stream_open: true,
              stream_state: sorp.stream_state,
              waker: local_cx.waker().clone(),
              windows: sorp.windows,
            },
          ));
          *method
        },
      )
    })
    .await;
    if let Err(err) = &rslt {
      process_higher_operation_err(err, hd).await;
    }
    rslt
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  #[inline]
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    send_go_away(error_code, &mut self.hd.lock().await.parts_mut()).await;
  }

  /// Send Response
  ///
  /// Higher operation that sends all data related to a response and then closes the stream.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Should be called after [`Self::recv_req`] is successfully executed.
  #[inline]
  pub async fn send_res<RRD>(&mut self, res: Response<RRD>) -> crate::Result<Option<()>>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending response");
    if send_msg::<_, _, _, _, false>(
      res.rrd.body().lease(),
      &self.hd,
      res.rrd.headers(),
      (
        HpackStaticRequestHeaders::EMPTY,
        HpackStaticResponseHeaders { status_code: Some(res.status_code) },
      ),
      &self.is_conn_open,
      self.stream_id,
      |_| {},
    )
    .await?
    .is_none()
    {
      return Ok(None);
    }
    sleep(Duration::from_millis(50)).await?;
    drop(self.hd.lock().await.parts_mut().hb.scrp.remove(&self.stream_id));
    Ok(Some(()))
  }

  /// Sends a stream reset to the peer, which cancels this stream.
  #[inline]
  pub async fn send_reset(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    let _ = send_reset_stream(
      error_code,
      &mut hdpm.hb.scrp,
      &mut hdpm.hb.sorp,
      hdpm.stream_writer,
      self.stream_id,
    )
    .await;
  }
}
