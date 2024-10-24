use crate::{
  http::{Headers, StatusCode},
  http2::{
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{check_content_length, frame_reader_rslt, sorp_mut, status_recv, status_send},
    send_data_mode::SendDataModeBytes,
    send_msg::{
      encode_headers, write_standalone_data, write_standalone_headers, write_standalone_trailers,
    },
    u31::U31,
    window::WindowsPair,
    Http2Buffer, Http2Data, Http2RecvStatus, Http2SendStatus, SendDataMode,
  },
  misc::{LeaseMut, Lock, RefCounter, StreamWriter, Vector, _Span},
};
use alloc::sync::Arc;
use core::{
  future::{poll_fn, Future},
  mem,
  pin::pin,
  sync::atomic::AtomicBool,
  task::{ready, Poll},
};

/// Groups common client and server operations as well as low level methods that deal with
/// individual frames.
#[derive(Debug)]
pub struct CommonStream<'instance, HD, const IS_CLIENT: bool> {
  pub(crate) hd: &'instance mut HD,
  pub(crate) is_conn_open: &'instance Arc<AtomicBool>,
  pub(crate) span: &'instance mut _Span,
  pub(crate) stream_id: U31,
}

impl<'instance, HB, HD, SW, const IS_CLIENT: bool> CommonStream<'instance, HD, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SW: StreamWriter,
{
  /// Removes internal elements that are no longer necessary after the end of the stream.
  ///
  /// If `linger` is true, then the stream will remain alive for a short period of time to allow
  /// the possible receiving of control frames.
  #[inline]
  pub async fn clear(&self, linger: bool) -> crate::Result<()> {
    if linger {
      crate::misc::sleep(core::time::Duration::from_millis(50)).await?;
    }
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    if let Some(elem) = hdpm.hb.scrp.remove(&self.stream_id) {
      elem.waker.wake();
    }
    if let Some(elem) = hdpm.hb.sorp.remove(&self.stream_id) {
      elem.waker.wake();
    }
    Ok(())
  }

  /// Receive Data
  ///
  /// Low level operation that retrieves a DATA frame sent by the remote peer. Shouldn't interact
  /// with higher operations.
  ///
  /// Returns [`Http2Status::Ok`] with `true` if no more data needs to be fetched.
  #[inline]
  pub async fn recv_data(&mut self) -> crate::Result<Http2RecvStatus<Vector<u8>, Vector<u8>>> {
    let _e = self.span._enter();
    _trace!("Fetching data");
    let mut pin = pin!(self.hd.lock());
    poll_fn(|cx| {
      let mut lock = lock_pin!(cx, self.hd, pin);
      let hdpm = lock.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
      if let Some(elem) = status_recv(&self.is_conn_open, sorp, |local_sorp| {
        check_content_length(&local_sorp)?;
        Ok(mem::take(&mut local_sorp.rrb.body))
      })? {
        return Poll::Ready(Ok(elem));
      }
      if sorp.has_one_or_more_data_frames && !sorp.rrb.body.is_empty() {
        frame_reader_rslt(hdpm.frame_reader_error)?;
        let rslt = sorp.rrb.body.clone();
        sorp.rrb.body.clear();
        Poll::Ready(Ok(Http2RecvStatus::Ongoing(rslt)))
      } else {
        sorp.waker.clone_from(cx.waker());
        Poll::Pending
      }
    })
    .await
  }

  /// Receive Trailers
  ///
  /// Low level operation that retrieves one or more frames that compose a header. Shouldn't interact
  /// with higher operations.
  #[inline]
  pub async fn recv_trailers(&mut self) -> crate::Result<Http2RecvStatus<Headers, ()>> {
    let _e = self.span._enter();
    _trace!("Fetching trailers");
    let mut pin = pin!(self.hd.lock());
    poll_fn(|cx| {
      let mut lock = lock_pin!(cx, self.hd, pin);
      let hdpm = lock.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
      if let Some(elem) = status_recv(&self.is_conn_open, sorp, |local_sorp| {
        Ok(mem::take(&mut local_sorp.rrb.headers))
      })? {
        return Poll::Ready(Ok(elem));
      }
      sorp.waker.clone_from(cx.waker());
      frame_reader_rslt(hdpm.frame_reader_error)?;
      Poll::Pending
    })
    .await
  }

  /// Should be used when receiving data to re-evaluated flow control values. Both connection and
  /// stream capacities are modified.
  ///
  /// `value` is capped to an integer of 31 bits.
  #[inline]
  pub async fn release_capacity(&mut self, value: u32) -> crate::Result<()> {
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let elem = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
    let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
    wp.withdrawn_recv(
      hdpm.hp,
      &self.is_conn_open,
      hdpm.stream_writer,
      self.stream_id,
      U31::from_u32(value),
    )
    .await
  }

  /// Should be used when sending data to re-evaluated flow control values. Both connection and
  /// stream capacities are modified.
  ///
  /// `value` is capped to an integer of 31 bits.
  #[inline]
  pub async fn reserve_capacity(&mut self, value: u32) -> crate::Result<()> {
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let elem = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
    let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
    wp.withdrawn_send(Some(self.stream_id), U31::from_u32(value))
  }

  /// Low level operation that sends the content of `data` as one or more DATA frames. If `eos` is
  /// true, then the last frame is set with the end-of-stream flag. Shouldn't interact with
  /// [`Self::send_res`].
  ///
  /// This method will spin until the entirety of `data` is sent and such behavior depends on the
  /// current available window size as well as the negotiated maximum frame length.
  #[inline]
  pub async fn send_data<'bytes, B, const IS_SCATTERED: bool>(
    &mut self,
    mut data: SendDataMode<B, IS_SCATTERED>,
    is_eos: bool,
  ) -> crate::Result<Http2SendStatus>
  where
    B: SendDataModeBytes<'bytes, IS_SCATTERED>,
  {
    let _e = self.span._enter();
    _trace!("Sending data");
    let mut has_data = false;
    let mut pin = pin!(self.hd.lock());
    poll_fn(|cx| {
      let mut lock = lock_pin!(cx, self.hd, pin);
      let hdpm = lock.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
      if let Some(elem) = status_send::<false>(&self.is_conn_open, sorp) {
        return Poll::Ready(Ok(elem));
      }
      let mut wp = WindowsPair::new(hdpm.windows, &mut sorp.windows);
      let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
        cx.waker().wake_by_ref();
        return Poll::Pending;
      };
      let fut = write_standalone_data(
        available_send,
        &mut data,
        is_eos,
        &mut has_data,
        false,
        &self.is_conn_open,
        hdpm.hps.max_frame_len,
        hdpm.stream_writer,
        self.stream_id,
        &mut wp,
      );
      let _ = ready!(pin!(fut).poll(cx))?;
      if has_data {
        Poll::Ready(Ok(Http2SendStatus::Ok))
      } else {
        cx.waker().wake_by_ref();
        Poll::Pending
      }
    })
    .await
  }

  send_go_away_method!();

  /// Low level operation that sends the content of `headers` with at most two frames. If `is_eos`
  /// is true, then the last frame is set with the end-of-stream flag. Shouldn't interact with
  /// [`Self::send_res`].
  ///
  /// If two frames aren't enough for the contents of `headers`, try increasing the maximum frame
  /// length.
  #[inline]
  pub async fn send_headers(
    &mut self,
    headers: &Headers,
    is_eos: bool,
    status_code: StatusCode,
  ) -> crate::Result<Http2SendStatus> {
    let _e = self.span._enter();
    _trace!("Sending headers");
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    let sorp = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
    if let Some(elem) = status_send::<false>(&self.is_conn_open, sorp) {
      return Ok(elem);
    }
    let hsreh = HpackStaticResponseHeaders { status_code: Some(status_code) };
    encode_headers::<false>(
      headers,
      (&mut hdpm.hb.hpack_enc, &mut hdpm.hb.hpack_enc_buffer),
      (HpackStaticRequestHeaders::EMPTY, hsreh),
    )?;
    let _ = write_standalone_headers::<_, IS_CLIENT>(
      &mut hdpm.hb.hpack_enc_buffer,
      (HpackStaticRequestHeaders::EMPTY, hsreh),
      &self.is_conn_open,
      is_eos,
      hdpm.hps.max_frame_len,
      hdpm.stream_writer,
      self.stream_id,
    )
    .await?;
    Ok(Http2SendStatus::Ok)
  }

  /// Sends a reset frame to the peer, which cancels this stream.
  #[inline]
  pub async fn send_reset(&self, error_code: crate::http2::Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    let _ = crate::http2::misc::send_reset_stream(
      error_code,
      &mut hdpm.hb.scrp,
      &mut hdpm.hb.sorp,
      hdpm.stream_writer,
      self.stream_id,
    )
    .await;
  }

  /// Low level operation that sends headers that are preceded by DATA frames and then closes
  /// the stream. Shouldn't interact with [`Self::send_res`].
  ///
  /// An error will probably be returned if the end-of-stream flag was set in previous operations.
  ///
  /// Returns `false` if the stream is already closed.
  #[inline]
  pub async fn send_trailers(&mut self, trailers: &Headers) -> crate::Result<Http2SendStatus> {
    let _e = self.span._enter();
    _trace!("Sending {} trailers", trailers.headers_len());
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let sorp = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
    if let Some(elem) = status_send::<false>(&self.is_conn_open, sorp) {
      return Ok(elem);
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
    Ok(Http2SendStatus::Ok)
  }

  /// Stream ID
  #[inline]
  pub const fn stream_id(&self) -> u32 {
    self.stream_id.u32()
  }

  /// Low level operation that returns the current flow control parameters of the stream.
  #[inline]
  pub async fn windows(&self) -> crate::Result<crate::http2::Windows> {
    let mut lock = self.hd.lock().await;
    let hdpm = lock.parts_mut();
    let elem = sorp_mut(&mut hdpm.hb.sorp, self.stream_id)?;
    Ok(elem.windows)
  }
}
