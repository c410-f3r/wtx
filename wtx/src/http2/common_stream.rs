use crate::{
  collection::{ArrayVectorU8, Vector},
  http::{Headers, StatusCode},
  http2::{
    Http2Buffer, Http2Inner, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{
      check_content_length, frame_reader_rslt, sorp_mut, status_recv, status_send, write_array,
    },
    u31::U31,
    window::WindowsPair,
    write_functions::{encode_headers, push_data, push_headers, push_trailers, write_frames},
  },
  misc::{LeaseMut, Usize, span::Span},
  stream::StreamWriter,
  sync::Arc,
};
use core::{future::poll_fn, mem, pin::pin, task::Poll};

/// Groups common client and server operations as well as low level methods that deal with
/// individual frames.
#[derive(Debug)]
pub struct CommonStream<'instance, HB, SW, const IS_CLIENT: bool> {
  pub(crate) inner: &'instance Arc<Http2Inner<HB, SW, IS_CLIENT>>,
  pub(crate) span: &'instance Span,
  pub(crate) stream_id: U31,
}

impl<HB, SW, const IS_CLIENT: bool> CommonStream<'_, HB, SW, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  /// Removes internal elements that are no longer necessary after the end of the stream.
  ///
  /// If `linger` is true, then the stream will remain alive for a short period of time to allow
  /// the possible receiving of control frames.
  #[inline]
  pub async fn clear(&self, linger: bool) -> crate::Result<()> {
    let Self { inner, span: _, stream_id } = self;
    if linger {
      crate::misc::sleep(core::time::Duration::from_millis(50)).await?;
    }
    let mut hd_guard = inner.hd.lock().await;
    let hdpm = hd_guard.parts_mut();
    if let Some(elem) = hdpm.hb.scrps.remove(stream_id) {
      elem.waker.wake();
    }
    if let Some(elem) = hdpm.hb.sorps.remove(stream_id) {
      elem.waker.wake();
    }
    Ok(())
  }

  /// Receive Data
  ///
  /// Low level operation that retrieves a DATA frame sent by the remote peer. Shouldn't interact
  /// with higher operations that receive data.
  #[inline]
  pub async fn recv_data(&mut self) -> crate::Result<Http2RecvStatus<Vector<u8>, Vector<u8>>> {
    let Self { inner, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Fetching data");
    let mut hd_guard_pin = pin!(inner.hd.lock());
    poll_fn(|cx| {
      let mut hd_guard = lock_pin!(cx, inner.hd, hd_guard_pin);
      let hdpm = hd_guard.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
      if let Some(elem) = status_recv(&inner.is_conn_open, sorp, |local_sorp| {
        check_content_length(local_sorp.content_length, &local_sorp.rrb)?;
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
  /// with higher operations that receive data.
  #[inline]
  pub async fn recv_trailers(&mut self) -> crate::Result<Http2RecvStatus<Headers, ()>> {
    let Self { inner, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Fetching trailers");
    let mut hd_guard_pin = pin!(inner.hd.lock());
    poll_fn(|cx| {
      let mut hd_guard = lock_pin!(cx, inner.hd, hd_guard_pin);
      let hdpm = hd_guard.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
      if let Some(elem) = status_recv(&inner.is_conn_open, sorp, |local_sorp| {
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
    let Self { inner, span: _, stream_id } = self;
    let frame = {
      let mut hd_guard = inner.hd.lock().await;
      let hdpm = hd_guard.parts_mut();
      let elem = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
      let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
      wp.withdrawn_recv(hdpm.hp, *stream_id, U31::from_u32(value))?
    };
    write_array([&frame], &inner.is_conn_open, &mut inner.wd.lock().await.stream_writer).await?;
    Ok(())
  }

  /// Should be used when sending data to re-evaluated flow control values. Both connection and
  /// stream capacities are modified.
  ///
  /// `value` is capped to an integer of 31 bits.
  #[inline]
  pub async fn reserve_capacity(&mut self, value: u32) -> crate::Result<()> {
    let Self { inner, span: _, stream_id } = self;
    let mut lock = inner.hd.lock().await;
    let hdpm = lock.parts_mut();
    let elem = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
    let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);
    wp.withdrawn_send(Some(*stream_id), U31::from_u32(value))
  }

  /// Low level operation that sends the content of `data` as one or more DATA frames. If `eos` is
  /// true, then the last frame is set with the end-of-stream flag. Shouldn't interact with
  /// higher operations that send data.
  ///
  /// This method will spin until the entirety of `data` is sent and such behavior depends on the
  /// current available window size as well as the negotiated maximum frame length.
  #[inline]
  pub async fn send_data(&self, data: &[u8], is_eos: bool) -> crate::Result<Http2SendStatus> {
    let Self { inner, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Sending data");
    let mut data_idx = 0;
    let mut frames = ArrayVectorU8::new();
    loop {
      let opt = {
        frames.clear();
        let mut hd_pin = pin!(inner.hd.lock());
        poll_fn(|cx| {
          let mut hd_guard = lock_pin!(cx, inner.hd, hd_pin);
          let hdpm = hd_guard.parts_mut();
          let sorp = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
          if let Some(elem) = status_send::<false>(&inner.is_conn_open, sorp) {
            return Poll::Ready(crate::Result::Ok(Some(elem)));
          }
          let mut wp = WindowsPair::new(hdpm.windows, &mut sorp.windows);
          let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
            sorp.waker.clone_from(cx.waker());
            return Poll::Pending;
          };
          let _ = push_data(
            available_send,
            data,
            &mut data_idx,
            &mut frames,
            is_eos,
            hdpm.hps.max_frame_len,
            *stream_id,
            &mut wp,
          )?;
          Poll::Ready(Ok(None))
        })
        .await?
      };
      match opt {
        Some(el) => return Ok(el),
        None => {
          write_frames(
            (&[], data),
            &frames,
            &inner.is_conn_open,
            &mut inner.wd.lock().await.stream_writer,
          )
          .await?;
          if *Usize::from(data_idx) >= data.len() {
            return Ok(Http2SendStatus::Ok);
          }
        }
      }
    }
  }

  send_go_away_method!();

  /// Low level operation that sends the content of `headers` with at most two frames. If `is_eos`
  /// is true, then the last frame is set with the end-of-stream flag. Shouldn't interact with
  /// higher operations that send data.
  ///
  /// If two frames aren't enough for the contents of `headers`, try increasing the maximum frame
  /// length.
  #[inline]
  pub async fn send_headers(
    &mut self,
    enc_buffer: &mut Vector<u8>,
    headers: &Headers,
    is_eos: bool,
    status_code: StatusCode,
  ) -> crate::Result<Http2SendStatus> {
    let Self { inner, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Sending headers");
    let hsreh = HpackStaticResponseHeaders { status_code: Some(status_code) };
    let max_frame_len = {
      let mut hd_guard = inner.hd.lock().await;
      let hdpm = hd_guard.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
      if let Some(elem) = status_send::<false>(&inner.is_conn_open, sorp) {
        return Ok(elem);
      }
      encode_headers::<false>(
        enc_buffer,
        headers,
        &mut hdpm.hb.hpack_enc,
        (HpackStaticRequestHeaders::EMPTY, hsreh),
      )?;
      hdpm.hp.max_frame_len()
    };
    let mut frames = ArrayVectorU8::new();
    let _ = push_headers::<IS_CLIENT>(
      enc_buffer,
      &mut frames,
      &mut 0,
      (HpackStaticRequestHeaders::EMPTY, hsreh),
      is_eos,
      max_frame_len,
      *stream_id,
    )?;
    write_frames(
      (&enc_buffer, &[]),
      &frames,
      &inner.is_conn_open,
      &mut inner.wd.lock().await.stream_writer,
    )
    .await?;
    Ok(Http2SendStatus::Ok)
  }

  /// Sends a reset frame to the peer, which cancels this stream.
  #[inline]
  pub async fn send_reset(&self, error_code: crate::http2::Http2ErrorCode) {
    let Self { inner, span: _, stream_id } = self;
    let _ = crate::http2::misc::send_reset_stream(error_code, inner, *stream_id).await;
  }

  /// Low level operation that sends headers that are preceded by DATA frames and then closes
  /// the stream. Shouldn't interact with higher operations that send data.
  ///
  /// An error will probably be returned if the end-of-stream flag was set in previous operations.
  ///
  /// Returns `false` if the stream is already closed.
  #[inline]
  pub async fn send_trailers(
    &mut self,
    enc_buffer: &mut Vector<u8>,
    trailers: &Headers,
  ) -> crate::Result<Http2SendStatus> {
    let Self { inner, span, stream_id } = self;
    let _e = span.enter();
    _trace!("Sending {} trailers", trailers.headers_len());
    let mut frames = ArrayVectorU8::new();
    {
      let mut hd_guard = inner.hd.lock().await;
      let hdpm = hd_guard.parts_mut();
      let sorp = sorp_mut(&mut hdpm.hb.sorps, *stream_id)?;
      if let Some(elem) = status_send::<false>(&inner.is_conn_open, sorp) {
        return Ok(elem);
      }
      let _ = push_trailers(
        enc_buffer,
        &mut frames,
        trailers,
        &mut hdpm.hb.hpack_enc,
        &mut 0,
        hdpm.hps.max_frame_len,
        *stream_id,
      )?;
    }
    write_frames(
      (enc_buffer, &[]),
      &frames,
      &inner.is_conn_open,
      &mut inner.wd.lock().await.stream_writer,
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
    let Self { inner, span: _, stream_id } = self;
    Ok(sorp_mut(&mut inner.hd.lock().await.parts_mut().hb.sorps, *stream_id)?.windows)
  }
}
