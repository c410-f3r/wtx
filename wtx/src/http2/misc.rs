use crate::{
  http2::{
    http2_data::Http2DataPartsMut, http2_params_send::Http2ParamsSend, CommonFlags, FrameInit,
    FrameInitTy, GoAwayFrame, HeadersFrame, HpackDecoder, HpackEncoder, Http2Buffer, Http2Data,
    Http2Error, Http2ErrorCode, Http2Params, PingFrame, ReqResBuffer, ResetStreamFrame, Scrp,
    SettingsFrame, Sorp, StreamBuffer, StreamOverallRecvParams, StreamState, UriBuffer,
    WindowUpdateFrame, Windows, U31,
  },
  misc::{
    LeaseMut, PartitionedFilledBuffer, PollOnce, Stream, Usize, _read_until, atoi, Lock, RefCounter,
  },
};
use core::pin::pin;
use tokio::sync::MutexGuard;

#[inline]
pub(crate) fn apply_initial_params<SB>(
  hb: &mut Http2Buffer<SB>,
  hp: &Http2Params,
) -> crate::Result<()> {
  hb.hpack_dec.set_max_bytes(hp.max_hpack_len().0);
  hb.hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
  hb.pfb._expand_buffer(*Usize::from(hp.read_buffer_len()));
  Ok(())
}

#[inline]
pub(crate) fn check_content_length<SB>(
  headers_idx: usize,
  sorp: &StreamOverallRecvParams<SB>,
) -> crate::Result<()>
where
  SB: LeaseMut<StreamBuffer>,
{
  if let Some(header) = sorp.sb.lease().rrb.headers.get_by_idx(headers_idx) {
    if sorp.sb.lease().rrb.body.len() != atoi::<usize>(header.value)? {
      return Err(protocol_err(Http2Error::InvalidHeaderData));
    }
  }
  Ok(())
}

pub(crate) const fn protocol_err(error: Http2Error) -> crate::Error {
  crate::Error::Http2ErrorGoAway(Http2ErrorCode::ProtocolError, Some(error))
}

#[inline]
pub(crate) async fn process_higher_operation_err<HB, HD, S, SB, const IS_CLIENT: bool>(
  err: crate::Error,
  hd: &HD,
) -> crate::Error
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, SB, IS_CLIENT>>,
      Resource = Http2Data<HB, S, SB, IS_CLIENT>,
    > + 'guard,
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  #[inline]
  pub(crate) async fn send_based_on_error<S, SB>(
    err: &crate::Error,
    hdpm: Http2DataPartsMut<'_, S, SB>,
  ) where
    SB: LeaseMut<StreamBuffer>,
    S: Stream,
  {
    match err {
      crate::Error::Http2ErrorGoAway(http2_error_code, _) => {
        send_go_away(*http2_error_code, hdpm.is_conn_open, *hdpm.last_stream_id, hdpm.stream).await;
      }
      crate::Error::Http2ErrorReset(http2_error_code, _, stream_id) => {
        send_reset_stream(*http2_error_code, hdpm.hb, hdpm.stream, stream_id.into()).await;
      }
      _ => {
        send_go_away(
          Http2ErrorCode::InternalError,
          hdpm.is_conn_open,
          *hdpm.last_stream_id,
          hdpm.stream,
        )
        .await;
      }
    }
  }

  let now = crate::misc::GenericTime::now();
  let mut idx: u8 = 0;
  let mut has_reset_err = false;
  loop {
    let mut guard = hd.lock().await;
    if idx >= crate::http2::MAX_FINAL_FETCHES {
      send_based_on_error(&err, guard.parts_mut()).await;
      return err;
    }
    has_reset_err |= matches!(&err, crate::Error::Http2ErrorReset(..));
    let local_rslt = guard.process_receipt().await;
    let hdpm = guard.parts_mut();
    if has_reset_err {
      if let Err(local_err) = local_rslt {
        send_based_on_error(&local_err, hdpm).await;
        return local_err;
      }
    }
    if now.elapsed().ok().map_or(true, |el| el >= crate::http2::MAX_FINAL_DURATION) {
      send_based_on_error(&err, hdpm).await;
      return err;
    }
    idx = idx.wrapping_add(1);
  }
}

#[inline]
pub(crate) async fn read_frame<S, const IS_HEADER_BLOCK: bool>(
  hp: &mut Http2Params,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<Option<FrameInit>>
where
  S: Stream,
{
  if !*is_conn_open {
    return Err(crate::Error::Http2ErrorGoAway(Http2ErrorCode::Cancel, None));
  }
  for _ in 0.._max_frames_mismatches!() {
    pfb._clear_if_following_is_empty();
    let mut read = pfb._following_len();
    let buffer = pfb._following_trail_mut();
    let Some(array_rslt) = PollOnce(pin!(_read_until::<9, _>(buffer, &mut read, 0, stream))).await
    else {
      return Ok(None);
    };
    let array = array_rslt?;
    let (fi_opt, data_len) = FrameInit::from_array(array);
    if data_len > hp.max_frame_len() {
      return Err(protocol_err(Http2Error::LargeArbitraryFrameLen));
    }
    let frame_len = *Usize::from_u32(data_len.wrapping_add(9));
    let Some(fi) = fi_opt else {
      if IS_HEADER_BLOCK {
        return Err(protocol_err(Http2Error::UnexpectedContinuationFrame));
      }
      if data_len > 32 {
        return Err(protocol_err(Http2Error::LargeIgnorableFrameLen));
      }
      let (antecedent_len, following_len) = if let Some(to_read) = frame_len.checked_sub(read) {
        stream.read_skip(to_read).await?;
        (pfb._buffer().len(), 0)
      } else {
        (pfb._current_end_idx().wrapping_add(frame_len), read.wrapping_sub(frame_len))
      };
      pfb._set_indices(antecedent_len, 0, following_len)?;
      continue;
    };
    _trace!("Received frame: {fi:?}");
    let mut is_fulfilled = false;
    pfb._expand_following(*Usize::from(data_len));
    for _ in 0..=data_len {
      if read >= frame_len {
        is_fulfilled = true;
        break;
      }
      read = read.wrapping_add(
        stream.read(pfb._following_trail_mut().get_mut(read..).unwrap_or_default()).await?,
      );
    }
    if !is_fulfilled {
      return Err(crate::Error::MISC_UnexpectedBufferState);
    }
    pfb._set_indices(
      pfb._current_end_idx().wrapping_add(9),
      *Usize::from(data_len),
      read.wrapping_sub(frame_len),
    )?;
    return Ok(Some(fi));
  }
  Err(protocol_err(Http2Error::VeryLargeAmountOfFrameMismatches))
}

/// Reads frames and return the first that is NOT related to the connection
#[inline]
pub(crate) async fn read_frame_until<S, SB>(
  conn_windows: &mut Windows,
  hp: &mut Http2Params,
  hpack_enc: &mut HpackEncoder,
  hps: &mut Http2ParamsSend,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  scrp: &mut Scrp,
  sorp: &mut Sorp<SB>,
  stream: &mut S,
) -> crate::Result<Option<FrameInit>>
where
  S: Stream,
{
  for _ in 0.._max_frames_mismatches!() {
    let Some(fi) = read_frame::<_, false>(hp, is_conn_open, pfb, stream).await? else {
      return Ok(None);
    };
    match fi.ty {
      FrameInitTy::GoAway => {
        let gaf = GoAwayFrame::read(pfb._current(), fi)?;
        return Err(crate::Error::Http2ErrorGoAway(gaf.error_code(), None));
      }
      FrameInitTy::Ping => {
        let mut pf = PingFrame::read(pfb._current(), fi)?;
        if !pf.has_ack() {
          pf.set_ack();
          write_array([&pf.bytes()], *is_conn_open, stream).await?;
        }
        continue;
      }
      FrameInitTy::Settings => {
        let sf = SettingsFrame::read(pfb._current(), fi)?;
        if !sf.has_ack() {
          hps.update(hpack_enc, scrp, &sf, sorp, conn_windows)?;
          write_array([SettingsFrame::ack().bytes(&mut [0; 45])], *is_conn_open, stream).await?;
        }
        continue;
      }
      FrameInitTy::WindowUpdate if fi.stream_id.is_zero() => {
        let wuf = WindowUpdateFrame::read(pfb._current(), fi)?;
        conn_windows.send.deposit(None, wuf.size_increment().i32())?;
        continue;
      }
      _ => {
        if fi.stream_id.is_zero() {
          return Err(protocol_err(Http2Error::FrameIsZeroButShouldNot));
        }
      }
    }
    return Ok(Some(fi));
  }
  Err(protocol_err(Http2Error::VeryLargeAmountOfFrameMismatches))
}

#[inline]
pub(crate) async fn read_header_and_continuations<
  H,
  S,
  const IS_CLIENT: bool,
  const IS_TRAILER: bool,
>(
  fi: FrameInit,
  hp: &mut Http2Params,
  hpack_dec: &mut HpackDecoder,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  rrb: &mut ReqResBuffer,
  stream: &mut S,
  uri_buffer: &mut UriBuffer,
  mut headers_cb: impl FnMut(&HeadersFrame<'_>) -> crate::Result<H>,
) -> crate::Result<(Option<usize>, bool, H)>
where
  S: Stream,
{
  if IS_TRAILER && !fi.cf.has_eos() {
    return Err(protocol_err(Http2Error::MissingEOSInTrailer));
  }

  if fi.cf.has_eoh() {
    let (content_length_idx, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
      pfb._current(),
      fi,
      &mut rrb.headers,
      hp,
      hpack_dec,
      &mut rrb.uri,
      uri_buffer,
    )?;
    if hf.is_over_size() {
      return Err(protocol_err(Http2Error::VeryLargeHeadersLen));
    }
    return Ok((content_length_idx, hf.has_eos(), headers_cb(&hf)?));
  }

  rrb.body.clear();
  rrb.body.reserve(pfb._current().len());
  rrb.body.extend_from_slice(pfb._current())?;

  'continuation_frames: {
    for _ in 0.._max_continuation_frames!() {
      let frame_fi = loop_until_some!(read_frame::<_, true>(hp, is_conn_open, pfb, stream).await?);
      let has_diff_id = fi.stream_id != frame_fi.stream_id;
      let is_not_continuation = frame_fi.ty != FrameInitTy::Continuation;
      if has_diff_id || is_not_continuation {
        return Err(protocol_err(Http2Error::UnexpectedContinuationFrame));
      }
      rrb.body.reserve(pfb._current().len());
      rrb.body.extend_from_slice(pfb._current())?;
      if frame_fi.cf.has_eoh() {
        break 'continuation_frames;
      }
    }
    return Err(protocol_err(Http2Error::VeryLargeAmountOfContinuationFrames));
  }

  let (content_length_idx, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
    &rrb.body,
    fi,
    &mut rrb.headers,
    hp,
    hpack_dec,
    &mut rrb.uri,
    uri_buffer,
  )?;
  if hf.is_over_size() {
    return Err(protocol_err(Http2Error::VeryLargeHeadersLen));
  }
  rrb.body.clear();
  Ok((content_length_idx, hf.has_eos(), headers_cb(&hf)?))
}

#[inline]
pub(crate) async fn send_go_away<S>(
  error_code: Http2ErrorCode,
  is_conn_open: &mut bool,
  last_stream_id: U31,
  stream: &mut S,
) where
  S: Stream,
{
  *is_conn_open = false;
  let _rslt = stream.write_all(&GoAwayFrame::new(error_code, last_stream_id).bytes()).await;
}

#[inline]
pub(crate) async fn send_reset_stream<S, SB>(
  error_code: Http2ErrorCode,
  hb: &mut Http2Buffer<SB>,
  stream: &mut S,
  stream_id: U31,
) where
  S: Stream,
{
  let _opt = hb.scrp.remove(&stream_id);
  let _opt = hb.sorp.remove(&stream_id);
  let _rslt = stream.write_all(&ResetStreamFrame::new(error_code, stream_id).bytes()).await;
}

#[inline]
pub(crate) fn server_header_stream_state(is_eos: bool) -> StreamState {
  if is_eos {
    StreamState::HalfClosedRemote
  } else {
    StreamState::Open
  }
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
pub(crate) async fn write_array<S, const N: usize>(
  array: [&[u8]; N],
  is_conn_open: bool,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  if !is_conn_open {
    return Ok(());
  }
  _trace!("Sending frame(s): {:?}", {
    let mut is_prev_init = false;
    let mut rslt = [None; N];
    for (elem, frame) in rslt.iter_mut().zip(array.iter()) {
      if let ([a, b, c, d, e, f, g, h, i], false) = (frame, is_prev_init) {
        if let (Some(frame_init), _) = FrameInit::from_array([*a, *b, *c, *d, *e, *f, *g, *h, *i]) {
          is_prev_init = true;
          *elem = Some(frame_init);
        }
      } else {
        is_prev_init = false;
      }
    }
    rslt
  });
  stream.write_all_vectored(array).await?;
  Ok(())
}
