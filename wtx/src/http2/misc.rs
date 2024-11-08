use crate::{
  http::ReqResBuffer,
  http2::{
    common_flags::CommonFlags,
    frame_init::{FrameInit, FrameInitTy},
    go_away_frame::GoAwayFrame,
    headers_frame::HeadersFrame,
    hpack_decoder::HpackDecoder,
    http2_data::Http2DataPartsMut,
    reset_stream_frame::ResetStreamFrame,
    stream_receiver::{StreamControlRecvParams, StreamOverallRecvParams},
    stream_state::StreamState,
    u31::U31,
    uri_buffer::UriBuffer,
    Http2Buffer, Http2Data, Http2Error, Http2ErrorCode, Http2Params, Http2RecvStatus,
    Http2SendStatus, Scrp, Sorp,
  },
  misc::{
    AtomicWaker, LeaseMut, Lock, PartitionedFilledBuffer, RefCounter, StreamReader, StreamWriter,
    Usize, _read_header, _read_payload,
  },
};
use core::{
  future::{poll_fn, Future},
  pin::pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{ready, Context, Poll},
};

#[inline]
pub(crate) fn check_content_length(sorp: &StreamOverallRecvParams) -> crate::Result<()> {
  let Some(content_length) = sorp.content_length else {
    return Ok(());
  };
  if sorp.rrb.body.len() != content_length {
    return Err(protocol_err(Http2Error::InvalidContentLength));
  }
  Ok(())
}

#[inline]
pub(crate) fn frame_reader_rslt(err: &mut Option<crate::Error>) -> crate::Result<()> {
  match err.take() {
    Some(elem) => Err(elem),
    None => Ok(()),
  }
}

#[inline]
#[track_caller]
pub(crate) fn scrp_mut(
  scrp: &mut Scrp,
  stream_id: U31,
) -> crate::Result<&mut StreamControlRecvParams> {
  scrp.get_mut(&stream_id).ok_or_else(|| protocol_err(Http2Error::UnknownStreamId))
}

#[inline]
#[track_caller]
pub(crate) fn sorp_mut(
  sorp: &mut Sorp,
  stream_id: U31,
) -> crate::Result<&mut StreamOverallRecvParams> {
  sorp.get_mut(&stream_id).ok_or_else(|| protocol_err(Http2Error::UnknownStreamId))
}

#[inline]
pub(crate) fn manage_initial_stream_receiving(
  is_conn_open: &AtomicBool,
  rrb: &mut ReqResBuffer,
) -> bool {
  if !is_conn_open.load(Ordering::Relaxed) {
    return false;
  }
  rrb.clear();
  true
}

#[inline]
pub(crate) fn manage_recurrent_stream_receiving<E, SW, const IS_CLIENT: bool>(
  cx: &mut Context<'_>,
  mut hdpm: Http2DataPartsMut<'_, SW, IS_CLIENT>,
  is_conn_open: &AtomicBool,
  stream_id: U31,
  cb: impl FnOnce(
    &mut Context<'_>,
    &mut Http2DataPartsMut<'_, SW, IS_CLIENT>,
    &StreamOverallRecvParams,
  ) -> E,
) -> Poll<crate::Result<(Http2RecvStatus<E, ()>, ReqResBuffer)>> {
  let sorp = sorp_mut(&mut hdpm.hb.sorp, stream_id)?;
  'block: {
    let (hrs, rrb_opt) = match (is_conn_open.load(Ordering::Relaxed), sorp.is_stream_open) {
      (false, false) => {
        if let Some(elem) = hdpm.hb.scrp.remove(&stream_id) {
          elem.waker.wake();
        }
        let rrb_opt = hdpm.hb.sorp.remove(&stream_id).map(|el| {
          el.waker.wake();
          el.rrb
        });
        (Http2RecvStatus::ClosedConnection, rrb_opt)
      }
      (false, true) => {
        let rrb_opt = hdpm.hb.sorp.remove(&stream_id).map(|el| {
          el.waker.wake();
          el.rrb
        });
        (Http2RecvStatus::ClosedConnection, rrb_opt)
      }
      (true, false) => {
        if let Some(elem) = hdpm.hb.scrp.remove(&stream_id) {
          elem.waker.wake();
        }
        let rrb_opt = hdpm.hb.sorp.remove(&stream_id).map(|el| {
          el.waker.wake();
          el.rrb
        });
        (Http2RecvStatus::ClosedStream, rrb_opt)
      }
      (true, true) => {
        break 'block;
      }
    };
    if let Some(elem) = rrb_opt {
      frame_reader_rslt(hdpm.frame_reader_error)?;
      return Poll::Ready(Ok((hrs, elem)));
    }
    return Poll::Ready(Err(protocol_err(Http2Error::UnknownStreamReceiver)));
  }
  if sorp.stream_state.recv_eos() {
    if let Some(elem) = hdpm.hb.sorp.remove(&stream_id) {
      check_content_length(&elem)?;
      let rslt = cb(cx, &mut hdpm, &elem);
      return Poll::Ready(Ok((Http2RecvStatus::Eos(rslt), elem.rrb)));
    }
  } else {
    sorp.waker.clone_from(cx.waker());
  }
  Poll::Pending
}

#[inline]
pub(crate) const fn protocol_err(error: Http2Error) -> crate::Error {
  crate::Error::Http2ErrorGoAway(Http2ErrorCode::ProtocolError, Some(error))
}

#[inline]
pub(crate) async fn process_higher_operation_err<HB, HD, SW, const IS_CLIENT: bool>(
  err: &crate::Error,
  hd: &HD,
) where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SW: StreamWriter,
{
  let mut lock = hd.lock().await;
  let mut hdpm = lock.parts_mut();
  match err {
    crate::Error::Http2ErrorGoAway(http2_error_code, _) => {
      send_go_away(*http2_error_code, &mut hdpm).await;
    }
    crate::Error::Http2ErrorReset(http2_error_code, _, stream_id) => {
      let _ = send_reset_stream(
        *http2_error_code,
        &mut hdpm.hb.scrp,
        &mut hdpm.hb.sorp,
        hdpm.stream_writer,
        stream_id.into(),
      )
      .await;
    }
    _ => {
      send_go_away(Http2ErrorCode::InternalError, &mut hdpm).await;
    }
  }
}

#[inline]
pub(crate) async fn read_frame<SR, const IS_HEADER_BLOCK: bool>(
  is_conn_open: &AtomicBool,
  max_frame_len: u32,
  pfb: &mut PartitionedFilledBuffer,
  read_frame_waker: &AtomicWaker,
  stream_reader: &mut SR,
) -> crate::Result<Option<FrameInit>>
where
  SR: StreamReader,
{
  let mut fut = pin!(async move {
    for _ in 0.._max_frames_mismatches!() {
      pfb._clear_if_following_is_empty();
      pfb._reserve(9)?;
      let mut read = pfb._following_len();
      let buffer = pfb._following_rest_mut();
      let array = _read_header::<0, 9, _>(buffer, &mut read, stream_reader).await?;
      let (fi_opt, data_len) = FrameInit::from_array(array);
      if data_len > max_frame_len {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::FrameSizeError,
          Some(Http2Error::LargeArbitraryFrameLen),
        ));
      }
      let data_len_usize = *Usize::from_u32(data_len);
      let Some(fi) = fi_opt else {
        if IS_HEADER_BLOCK {
          return Err(protocol_err(Http2Error::UnexpectedContinuationFrame));
        }
        if data_len > 32 {
          return Err(protocol_err(Http2Error::LargeIgnorableFrameLen));
        }
        let frame_len = data_len_usize.wrapping_add(9);
        let (antecedent_len, following_len) = if let Some(to_read) = frame_len.checked_sub(read) {
          stream_reader.read_skip(to_read).await?;
          (pfb._buffer().len(), 0)
        } else {
          (pfb._current_end_idx().wrapping_add(frame_len), read.wrapping_sub(frame_len))
        };
        pfb._set_indices(antecedent_len, 0, following_len)?;
        continue;
      };
      _trace!("Received frame: {fi:?}");
      _read_payload((9, data_len_usize), pfb, &mut read, stream_reader).await?;
      return Ok(fi);
    }
    Err(protocol_err(Http2Error::VeryLargeAmountOfFrameMismatches))
  });
  poll_fn(|cx| {
    if !is_conn_open.load(Ordering::Relaxed) {
      return Poll::Ready(Ok(None));
    }
    read_frame_waker.register(cx.waker());
    let fi = ready!(fut.as_mut().poll(cx))?;
    Poll::Ready(Ok(Some(fi)))
  })
  .await
}

#[inline]
pub(crate) async fn read_header_and_continuations<
  H,
  SR,
  const IS_CLIENT: bool,
  const IS_TRAILER: bool,
>(
  fi: FrameInit,
  is_conn_open: &AtomicBool,
  hp: &mut Http2Params,
  hpack_dec: &mut HpackDecoder,
  pfb: &mut PartitionedFilledBuffer,
  read_frame_waker: &AtomicWaker,
  rrb: &mut ReqResBuffer,
  stream_reader: &mut SR,
  uri_buffer: &mut UriBuffer,
  mut headers_cb: impl FnMut(&HeadersFrame<'_>) -> crate::Result<H>,
) -> crate::Result<(Option<usize>, bool, H)>
where
  SR: StreamReader,
{
  if IS_TRAILER && !fi.cf.has_eos() {
    return Err(protocol_err(Http2Error::MissingEOSInTrailer));
  }

  let rrb_body_start = if IS_TRAILER {
    rrb.body.len()
  } else {
    rrb.clear();
    0
  };

  if fi.cf.has_eoh() {
    let (content_length, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
      Some(pfb._current()),
      fi,
      hp,
      hpack_dec,
      (rrb, rrb_body_start),
      uri_buffer,
    )?;

    if hf.is_over_size() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Some(Http2Error::VeryLargeHeadersLen),
      ));
    }
    return Ok((content_length, hf.has_eos(), headers_cb(&hf)?));
  }

  rrb.body.extend_from_copyable_slice(pfb._current())?;

  'continuation_frames: {
    for _ in 0.._max_continuation_frames!() {
      let Some(frame_fi) = read_frame::<_, true>(
        is_conn_open,
        hp.max_frame_len(),
        pfb,
        read_frame_waker,
        stream_reader,
      )
      .await?
      else {
        break 'continuation_frames;
      };
      let has_diff_id = fi.stream_id != frame_fi.stream_id;
      let is_not_continuation = frame_fi.ty != FrameInitTy::Continuation;
      if has_diff_id || is_not_continuation {
        return Err(protocol_err(Http2Error::UnexpectedContinuationFrame));
      }
      rrb.body.extend_from_copyable_slice(pfb._current())?;
      if frame_fi.cf.has_eoh() {
        break 'continuation_frames;
      }
    }
    return Err(protocol_err(Http2Error::VeryLargeAmountOfContinuationFrames));
  }

  let (content_length, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
    None,
    fi,
    hp,
    hpack_dec,
    (rrb, rrb_body_start),
    uri_buffer,
  )?;
  if IS_TRAILER {
    rrb.body.truncate(rrb_body_start);
  } else {
    rrb.clear();
  }
  if hf.is_over_size() {
    return Err(crate::Error::Http2ErrorGoAway(
      Http2ErrorCode::FrameSizeError,
      Some(Http2Error::VeryLargeHeadersLen),
    ));
  }
  Ok((content_length, hf.has_eos(), headers_cb(&hf)?))
}

#[inline]
pub(crate) async fn send_go_away<SW, const IS_CLIENT: bool>(
  error_code: Http2ErrorCode,
  hdpm: &mut Http2DataPartsMut<'_, SW, IS_CLIENT>,
) where
  SW: StreamWriter,
{
  hdpm.hb.is_conn_open.store(false, Ordering::Relaxed);
  let gaf = GoAwayFrame::new(error_code, *hdpm.last_stream_id);
  let _rslt = hdpm.stream_writer.write_all(&gaf.bytes()).await;
  for (_, value) in hdpm.hb.initial_server_headers.iter() {
    value.waker.wake_by_ref();
  }
  for scrp in hdpm.hb.scrp.values() {
    scrp.waker.wake_by_ref();
  }
  for sorp in hdpm.hb.sorp.values() {
    sorp.waker.wake_by_ref();
  }
  hdpm.hb.read_frame_waker.wake();
}

#[inline]
pub(crate) async fn send_reset_stream<SW>(
  error_code: Http2ErrorCode,
  scrp: &mut Scrp,
  sorp: &mut Sorp,
  stream_writer: &mut SW,
  stream_id: U31,
) -> bool
where
  SW: StreamWriter,
{
  let mut has_stored = false;
  let _rslt = stream_writer.write_all(&ResetStreamFrame::new(error_code, stream_id).bytes()).await;
  if let Some(elem) = scrp.get_mut(&stream_id) {
    has_stored = true;
    elem.is_stream_open = false;
    elem.stream_state = StreamState::Closed;
    elem.waker.wake_by_ref();
  }
  if let Some(elem) = sorp.get_mut(&stream_id) {
    has_stored = true;
    elem.is_stream_open = false;
    elem.stream_state = StreamState::Closed;
    elem.waker.wake_by_ref();
  }
  has_stored
}

#[inline]
pub(crate) fn server_header_stream_state(has_eos: bool) -> StreamState {
  if has_eos {
    StreamState::HalfClosedRemote
  } else {
    StreamState::Open
  }
}

#[inline]
pub(crate) fn status_recv<E, O>(
  is_conn_open: &AtomicBool,
  sorp: &mut StreamOverallRecvParams,
  eos_cb: impl FnOnce(&mut StreamOverallRecvParams) -> crate::Result<E>,
) -> crate::Result<Option<Http2RecvStatus<E, O>>> {
  if !is_conn_open.load(Ordering::Relaxed) {
    return Ok(Some(Http2RecvStatus::ClosedConnection));
  }
  if !sorp.is_stream_open {
    return Ok(Some(Http2RecvStatus::ClosedStream));
  }
  if sorp.stream_state.recv_eos() {
    return Ok(Some(Http2RecvStatus::Eos(eos_cb(sorp)?)));
  }
  Ok(None)
}

#[inline]
pub(crate) fn status_send<const IS_CLIENT: bool>(
  is_conn_open: &AtomicBool,
  sorp: &StreamOverallRecvParams,
) -> Option<Http2SendStatus> {
  if !is_conn_open.load(Ordering::Relaxed) {
    return Some(Http2SendStatus::ClosedConnection);
  }
  if !sorp.is_stream_open {
    return Some(Http2SendStatus::ClosedStream);
  }
  if !sorp.stream_state.can_send::<IS_CLIENT>() {
    return Some(Http2SendStatus::InvalidState);
  }
  None
}

#[inline]
pub(crate) fn trim_frame_pad(cf: CommonFlags, data: &mut &[u8]) -> crate::Result<Option<u8>> {
  let mut pad_len = None;
  if cf.has_pad() {
    let [local_pad_len, rest @ ..] = data else {
      return Err(protocol_err(Http2Error::InvalidFramePad));
    };
    let idx_opt = rest.len().checked_sub(usize::from(*local_pad_len));
    let Some(local_data) = idx_opt.and_then(|idx| rest.get(..idx)) else {
      return Err(protocol_err(Http2Error::InvalidFramePad));
    };
    *data = local_data;
    pad_len = Some(*local_pad_len);
  }
  Ok(pad_len)
}

#[inline]
pub(crate) async fn write_array<SW, const N: usize>(
  array: [&[u8]; N],
  is_conn_open: &AtomicBool,
  stream_writer: &mut SW,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  if !is_conn_open.load(Ordering::Relaxed) {
    return Ok(());
  }
  _trace!("Sending frame(s): {:?}", {
    let process = |elem: &mut Option<_>, frame: &[u8]| {
      let [a, b, c, d, e, f, g, h, i, rest @ ..] = frame else {
        return;
      };
      if rest.len() > 36 {
        return;
      }
      let (Some(fi), _) = FrameInit::from_array([*a, *b, *c, *d, *e, *f, *g, *h, *i]) else {
        return;
      };
      *elem = Some(fi);
    };
    let mut rslt = [None; N];
    let mut iter = rslt.iter_mut().zip(array.iter());
    if let Some((elem, frame)) = iter.next() {
      if frame != crate::http2::PREFACE {
        process(elem, frame);
      }
    }
    for (elem, frame) in iter {
      process(elem, frame);
    }
    rslt
  });
  stream_writer.write_all_vectored(&array).await?;
  Ok(())
}
