use crate::{
  http::{Headers, Method, ReqResBuffer, ReqResData, Response},
  http2::{
    misc::{manage_recurrent_stream_receiving, process_higher_operation_err, protocol_err},
    send_msg::{send_msg, write_standalone_data, write_standalone_trailers},
    window::WindowsPair,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2Error,
    Http2ErrorCode, Http2Status, StreamControlRecvParams, U31,
  },
  misc::{LeaseMut, Lock, RefCounter, StreamWriter, Vector, _Span, sleep, Lease},
};
use alloc::sync::Arc;
use core::{
  future::{poll_fn, Future},
  mem,
  pin::pin,
  sync::atomic::{AtomicBool, Ordering},
  task::Poll,
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

impl<HB, HD, HO, SW> ServerStream<HD>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, HO, SW, false>>,
  SW: StreamWriter,
{
  /// Low level operation that returns the current available flow control capacity of the
  /// connection as well as the stream.
  #[inline]
  pub async fn capacity(&self) -> crate::Result<(i32, i32)> {
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let Some(elem) = hdpm.hb.scrp.get_mut(&self.stream_id) else {
      return Err(protocol_err(Http2Error::BadLocalFlow));
    };
    let wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
    Ok((wp.conn.send.available(), wp.stream.send.available()))
  }

  /// Low level operation that retrieves a DATA frame sent by the remote peer. Shouldn't interact
  /// with [`Self::recv_req`].
  #[inline]
  pub async fn fetch_data(&mut self, mut body: Vector<u8>) -> crate::Result<(Vector<u8>, bool)> {
    let _e = self.span._enter();
    _trace!("Fetching data");
    body.clear();
    let mut body_opt = Some(body);
    let mut pin = pin!(self.hd.lock());
    poll_fn(|cx| {
      let mut lock = lock_pin!(cx, self.hd, pin);
      let hdpm = lock.parts_mut();
      let Some(elem) = hdpm.hb.sorp.get_mut(&self.stream_id) else {
        return Poll::Ready(Err(protocol_err(Http2Error::BadLocalFlow)));
      };
      if let Some(local_body) = body_opt.take() {
        elem.rrb.body = local_body;
        elem.waker.clone_from(cx.waker());
        Poll::Pending
      } else {
        Poll::Ready(Ok((mem::take(&mut elem.rrb.body), elem.is_stream_open)))
      }
    })
    .await
  }

  /// Low level operation that retrieves one or more frames that compose a header. Shouldn't interact
  /// with [`Self::recv_req`].
  #[inline]
  pub async fn fetch_trailers(
    &mut self,
    mut trailers: Headers,
  ) -> crate::Result<(Headers, Http2Status)> {
    let _e = self.span._enter();
    _trace!("Fetching trailers");
    trailers.clear();
    let mut pin = pin!(self.hd.lock());
    let mut trailers_opt = Some(trailers);
    poll_fn(|cx| {
      let mut lock = lock_pin!(cx, self.hd, pin);
      let hdpm = lock.parts_mut();
      let Some(elem) = hdpm.hb.sorp.get_mut(&self.stream_id) else {
        return Poll::Ready(Err(protocol_err(Http2Error::BadLocalFlow)));
      };
      if let Some(local_trailers) = trailers_opt.take() {
        if !self.is_conn_open.load(Ordering::Relaxed) {
          return Poll::Ready(Ok((local_trailers, Http2Status::ClosedConnection)));
        }
        if !elem.is_stream_open {
          return Poll::Ready(Ok((local_trailers, Http2Status::ClosedStream)));
        }
        if elem.stream_state.recv_eos() {
          return Poll::Ready(Ok((local_trailers, Http2Status::Ok)));
        }
        elem.rrb.headers = local_trailers;
        elem.waker.clone_from(cx.waker());
        Poll::Pending
      } else {
        let local_trailers = mem::take(&mut elem.rrb.headers);
        if !self.is_conn_open.load(Ordering::Relaxed) {
          return Poll::Ready(Ok((local_trailers, Http2Status::ClosedConnection)));
        }
        if !elem.is_stream_open {
          return Poll::Ready(Ok((local_trailers, Http2Status::ClosedStream)));
        }
        if elem.stream_state.recv_eos() {
          return Poll::Ready(Ok((local_trailers, Http2Status::Eos)));
        }
        Poll::Ready(Ok((local_trailers, Http2Status::Ok)))
      }
    })
    .await
  }

  /// See [`Method`].
  #[inline]
  pub fn method(&self) -> Method {
    self.method
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
  pub async fn recv_req(&mut self) -> crate::Result<(ReqResBuffer, bool)> {
    let Self { hd, is_conn_open, method: _, span, stream_id } = self;
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
        },
      )
    })
    .await;
    if let Err(err) = &rslt {
      process_higher_operation_err(err, hd).await;
    }
    let (req_res_buffer, opt) = rslt?;
    Ok((req_res_buffer, opt.is_some()))
  }

  /// Should be used when sending data to re-evaluated control flow values. Both connection and
  /// stream capacities are modified.
  ///
  /// `value` is capped to an integer of 31 bits.
  #[inline]
  pub async fn reserve_capacity(&mut self, value: u32) -> crate::Result<()> {
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let Some(elem) = hdpm.hb.scrp.get_mut(&self.stream_id) else {
      return Err(protocol_err(Http2Error::BadLocalFlow));
    };
    let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
    wp.withdrawn_send(Some(self.stream_id), U31::from_u32(value))
  }

  /// Low level operation that sends the content of `data` as one or more DATA frames. If `eos` is
  /// true, then the last frame is set with the end-of-stream flag. Shouldn't interact with
  /// [`Self::send_res`].
  ///
  /// This method will spin until the entirety of `data` is sent and such behavior depends on the
  /// current available window size as well as the negotiated maximum frame length.
  ///
  /// Returns `false` if the stream was closed.
  #[inline]
  pub async fn send_data(&mut self, mut data: &[u8], eos: bool) -> crate::Result<bool> {
    let _e = self.span._enter();
    _trace!("Sending data of {} bytes", data.len());
    let mut has_data = false;
    while !has_data {
      let mut lock = self.hd.lock().await;
      let hdpm = lock.parts_mut();
      let Some(elem) = hdpm.hb.scrp.get_mut(&self.stream_id) else {
        return Err(protocol_err(Http2Error::BadLocalFlow));
      };
      if !elem.is_stream_open {
        return Ok(false);
      }
      if !elem.stream_state.can_send_stream::<false>() {
        return Err(protocol_err(Http2Error::InvalidSendStreamState));
      }
      let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
      let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
        continue;
      };
      let _ = write_standalone_data(
        available_send,
        &mut data,
        eos,
        &mut has_data,
        false,
        &self.is_conn_open,
        hdpm.hps.max_frame_len,
        hdpm.stream_writer,
        self.stream_id,
        &mut wp,
      )
      .await?;
    }
    Ok(true)
  }

  send_go_away_method!();

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

  send_reset_method!();

  /// Low level operation that sends headers that are preceded by DATA frames and then closes
  /// the stream. Shouldn't interact with [`Self::send_res`].
  ///
  /// An error will probably be returned if the end-of-stream flag was set in previous operations.
  ///
  /// Returns `false` if the stream is already closed.
  #[inline]
  pub async fn send_trailers(&mut self, trailers: &Headers) -> crate::Result<bool> {
    let _e = self.span._enter();
    _trace!("Sending {} trailers", trailers.headers_len());
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let Some(elem) = hdpm.hb.scrp.get_mut(&self.stream_id) else {
      return Err(protocol_err(Http2Error::BadLocalFlow));
    };
    if !elem.is_stream_open {
      return Ok(false);
    }
    write_standalone_trailers(
      trailers,
      (&mut hdpm.hb.hpack_enc, &mut hdpm.hb.hpack_enc_buffer),
      &self.is_conn_open,
      hdpm.hps.max_frame_len,
      hdpm.stream_writer,
      self.stream_id,
    )
    .await?;
    Ok(true)
  }

  /// Stream ID
  #[inline]
  pub const fn stream_id(&self) -> u32 {
    self.stream_id.u32()
  }
}
