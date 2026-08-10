use crate::{
  collections::{SingleTypeStorage, Vector},
  http::{Method, MsgBufferString, MsgData, Protocol, Response, U31},
  http2::{
    CommonStream, Http2Inner, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{manage_recurrent_receiving_of_overall_stream, process_higher_operation_err},
    stream_receiver::StreamControlRecvParams,
    write_functions::send_msg,
  },
  misc::{Lease, LeaseMut, span::Span},
  net::StreamWriter,
  sync::Arc,
  tls::TlsCtx,
};
use core::{future::poll_fn, pin::pin, task::Waker};

/// Created when a server receives an initial stream.
#[derive(Debug)]
pub struct ServerStream<SW, TCX> {
  inner: Arc<Http2Inner<SW, TCX, false>>,
  linger: bool,
  method: Method,
  protocol: Option<Protocol>,
  span: Span,
  stream_id: U31,
}

impl<SW, TCX> ServerStream<SW, TCX>
where
  SW: StreamWriter,
  TCX: TlsCtx,
{
  pub(crate) const fn new(
    inner: Arc<Http2Inner<SW, TCX, false>>,
    linger: bool,
    method: Method,
    protocol: Option<Protocol>,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { inner, linger, method, protocol, span, stream_id }
  }

  /// See [`CommonStream`].
  #[inline]
  pub const fn common(&mut self) -> CommonStream<'_, SW, TCX, false> {
    let Self { inner, linger, method: _, protocol: _, span, stream_id } = self;
    CommonStream { inner, linger: *linger, span, stream_id: *stream_id }
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
  pub async fn recv_req(&mut self) -> crate::Result<(Http2RecvStatus<(), ()>, MsgBufferString)> {
    let Self { inner, linger: _, method: _, protocol: _, span, stream_id } = self;
    let _e = span.enter();
    _trace!(target: crate::_WTX_HTTP2, "Receiving request");
    let rslt = {
      let mut lock_pin = pin!(inner.hd.lock());
      poll_fn(|cx| {
        let mut lock = lock_pin!(cx, inner.hd, lock_pin);
        manage_recurrent_receiving_of_overall_stream(
          cx,
          lock.parts_mut(),
          &inner.is_conn_open.connection_state,
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
  pub async fn send_res<MD>(
    &mut self,
    enc_buffer: &mut Vector<u8>,
    res: Response<MD>,
  ) -> crate::Result<Http2SendStatus>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    let Self { inner, linger: _, method: _, protocol: _, span, stream_id } = self;
    let _e = span.enter();
    _trace!(target: crate::_WTX_HTTP2, "Sending response");
    let hss = send_msg::<_, _, false>(
      res.msg_data.body().lease(),
      enc_buffer,
      res.msg_data.headers(),
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

impl<SW, TCX> Lease<ServerStream<SW, TCX>> for ServerStream<SW, TCX> {
  #[inline]
  fn lease(&self) -> &ServerStream<SW, TCX> {
    self
  }
}

impl<SW, TCX> LeaseMut<ServerStream<SW, TCX>> for ServerStream<SW, TCX> {
  #[inline]
  fn lease_mut(&mut self) -> &mut ServerStream<SW, TCX> {
    self
  }
}

impl<SW, TCX> SingleTypeStorage for ServerStream<SW, TCX> {
  type Item = (SW, TCX);
}

impl<SW, TCX> Clone for ServerStream<SW, TCX> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      linger: self.linger,
      method: self.method,
      protocol: self.protocol,
      span: self.span.clone(),
      stream_id: self.stream_id,
    }
  }
}
