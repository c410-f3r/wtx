use crate::{
  http::{Method, Protocol, ReqResBuffer, ReqResData, Response},
  http2::{
    CommonStream, Http2Buffer, Http2Data, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{manage_recurrent_stream_receiving, process_higher_operation_err},
    send_msg::send_msg,
    stream_receiver::StreamControlRecvParams,
    u31::U31,
  },
  misc::{Lease, LeaseMut, SingleTypeStorage, span::Span},
  stream::StreamWriter,
  sync::{Arc, AtomicBool, Lock, RefCounter},
};
use core::{future::poll_fn, pin::pin};

/// Created when a server receives an initial stream.
#[derive(Clone, Debug)]
pub struct ServerStream<HD> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  method: Method,
  protocol: Option<Protocol>,
  span: Span,
  stream_id: U31,
}

impl<HD> ServerStream<HD> {
  pub(crate) const fn new(
    hd: HD,
    is_conn_open: Arc<AtomicBool>,
    method: Method,
    protocol: Option<Protocol>,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { hd, is_conn_open, method, protocol, span, stream_id }
  }
}

impl<HB, HD, SW> ServerStream<HD>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, false>>,
  SW: StreamWriter,
{
  /// See [`CommonStream`].
  #[inline]
  pub fn common(&mut self) -> CommonStream<'_, HD, false> {
    CommonStream {
      hd: &mut self.hd,
      is_conn_open: &self.is_conn_open,
      span: &mut self.span,
      stream_id: self.stream_id,
    }
  }

  /// See [`Method`].
  #[inline]
  pub fn method(&self) -> Method {
    self.method
  }

  /// See [`Protocol`].
  #[inline]
  pub fn protocol(&self) -> Option<Protocol> {
    self.protocol
  }

  /// Receive request
  ///
  /// High level operation that awaits for the data necessary to build a request.
  ///
  /// Returns `false` if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn recv_req(&mut self) -> crate::Result<(Http2RecvStatus<(), ()>, ReqResBuffer)> {
    let Self { hd, is_conn_open, method: _, protocol: _, span, stream_id } = self;
    let _e = span.enter();
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
        },
      )
    })
    .await;
    if let Err(err) = &rslt {
      process_higher_operation_err(err, hd).await;
    }
    rslt
  }

  /// Send Response
  ///
  /// Higher operation that sends all data related to a response and then closes the stream.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Should be called after [`Self::recv_req`] or any other low level methods that receive data
  /// are successfully executed. More specifically, should only be called in a half-closed stream
  /// state.
  #[inline]
  pub async fn send_res<RRD>(&mut self, res: Response<RRD>) -> crate::Result<Http2SendStatus>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let _e = self.span.enter();
    _trace!("Sending response");
    let hss = send_msg::<_, _, _, false>(
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
    .await?;
    if !matches!(hss, Http2SendStatus::ClosedConnection) {
      return Ok(hss);
    }
    Ok(Http2SendStatus::Ok)
  }
}

impl<HD> Lease<ServerStream<HD>> for ServerStream<HD> {
  #[inline]
  fn lease(&self) -> &ServerStream<HD> {
    self
  }
}

impl<HD> LeaseMut<ServerStream<HD>> for ServerStream<HD> {
  #[inline]
  fn lease_mut(&mut self) -> &mut ServerStream<HD> {
    self
  }
}

impl<HD> SingleTypeStorage for ServerStream<HD> {
  type Item = HD;
}
