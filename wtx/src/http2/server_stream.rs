use crate::{
  collection::Vector,
  http::{Method, Protocol, ReqResBuffer, ReqResData, Response},
  http2::{
    CommonStream, Http2Buffer, Http2Inner, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{manage_recurrent_receiving_of_overall_stream, process_higher_operation_err},
    stream_receiver::StreamControlRecvParams,
    u31::U31,
    write_functions::send_msg,
  },
  misc::{Lease, LeaseMut, SingleTypeStorage, span::Span},
  stream::StreamWriter,
  sync::Arc,
};
use core::{future::poll_fn, pin::pin, task::Waker};

/// Created when a server receives an initial stream.
#[derive(Debug)]
pub struct ServerStream<HB, SW> {
  inner: Arc<Http2Inner<HB, SW, false>>,
  method: Method,
  protocol: Option<Protocol>,
  span: Span,
  stream_id: U31,
}

impl<HB, SW> ServerStream<HB, SW>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  pub(crate) const fn new(
    inner: Arc<Http2Inner<HB, SW, false>>,
    method: Method,
    protocol: Option<Protocol>,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { inner, method, protocol, span, stream_id }
  }

  /// See [`CommonStream`].
  #[inline]
  pub const fn common(&mut self) -> CommonStream<'_, HB, SW, false> {
    let Self { inner, method: _, protocol: _, span, stream_id } = self;
    CommonStream { inner, span, stream_id: *stream_id }
  }

  /// See [`Method`].
  #[inline]
  pub const fn method(&self) -> Method {
    self.method
  }

  /// See [`Protocol`].
  #[inline]
  pub const fn protocol(&self) -> Option<Protocol> {
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
    let Self { inner, method: _, protocol: _, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Receiving request");
    let rslt = {
      let mut lock_pin = pin!(inner.hd.lock());
      poll_fn(|cx| {
        let mut lock = lock_pin!(cx, inner.hd, lock_pin);
        manage_recurrent_receiving_of_overall_stream(
          cx,
          lock.parts_mut(),
          &inner.is_conn_open,
          *stream_id,
          |hdpm, _, stream_state, windows| {
            drop(hdpm.hb.scrps.insert(
              *stream_id,
              StreamControlRecvParams {
                is_stream_open: true,
                stream_state,
                waker: Waker::noop().clone(),
                windows,
              },
            ));
          },
        )
      })
      .await
    };
    if let Err(err) = &rslt {
      process_higher_operation_err(err, inner).await;
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
  pub async fn send_res<RRD>(
    &mut self,
    enc_buffer: &mut Vector<u8>,
    res: Response<RRD>,
  ) -> crate::Result<Http2SendStatus>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let Self { inner, method: _, protocol: _, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Sending response");
    let hss = send_msg::<_, _, false>(
      res.rrd.body().lease(),
      enc_buffer,
      res.rrd.headers(),
      inner,
      (
        HpackStaticRequestHeaders::EMPTY,
        HpackStaticResponseHeaders { status_code: Some(res.status_code) },
      ),
      *stream_id,
      |_| {},
    )
    .await?;
    if !matches!(hss, Http2SendStatus::ClosedConnection) {
      return Ok(hss);
    }
    Ok(Http2SendStatus::Ok)
  }
}

impl<HB, SW> Lease<ServerStream<HB, SW>> for ServerStream<HB, SW> {
  #[inline]
  fn lease(&self) -> &ServerStream<HB, SW> {
    self
  }
}

impl<HB, SW> LeaseMut<ServerStream<HB, SW>> for ServerStream<HB, SW> {
  #[inline]
  fn lease_mut(&mut self) -> &mut ServerStream<HB, SW> {
    self
  }
}

impl<HB, SW> SingleTypeStorage for ServerStream<HB, SW> {
  type Item = (HB, SW);
}

impl<HB, SW> Clone for ServerStream<HB, SW> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      method: self.method.clone(),
      protocol: self.protocol.clone(),
      span: self.span.clone(),
      stream_id: self.stream_id.clone(),
    }
  }
}
