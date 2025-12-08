macro_rules! final_generic_calls {
  ($cb:ident, $hdpm:ident, $scrp:ident) => {{
    change_final_stream_state::<IS_CLIENT>(&mut $scrp.stream_state);
    $cb($hdpm);
  }};
}

macro_rules! frame_params {
  ($cursor:expr, $frame:expr, $frame_len:expr, $is_data:expr) => {{
    let mut header = [0; 9];
    let [a, b, c, d, e, f, g, h, i] = &mut header;
    let [_, j, k, l] = $frame_len.to_be_bytes();
    let [_, _, _, m, n, o, p, q, r] = $frame.bytes();
    *a = j;
    *b = k;
    *c = l;
    *d = m;
    *e = n;
    *f = o;
    *g = p;
    *h = q;
    *i = r;
    let begin = *$cursor;
    *$cursor = $cursor.wrapping_add($frame_len);
    FrameParams { header, is_data: $is_data, range: [begin, *$cursor] }
  }};
}

use crate::{
  collection::{ArrayVectorU8, Vector},
  http::{Headers, Trailers},
  http2::{
    Http2Buffer, Http2Data, Http2Error, Http2Inner, Http2SendStatus,
    continuation_frame::ContinuationFrame,
    data_frame::DataFrame,
    headers_frame::HeadersFrame,
    hpack_encoder::HpackEncoder,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    http2_data::Http2DataPartsMut,
    misc::{connection_state, process_higher_operation_err, protocol_err, scrp_mut, write_array},
    stream_state::StreamState,
    u31::U31,
    window::WindowsPair,
  },
  misc::{LeaseMut, Usize},
  stream::StreamWriter,
  sync::AtomicBool,
};
use core::{
  future::poll_fn,
  pin::pin,
  task::{Poll, Waker},
};

pub(crate) fn encode_headers<const IS_CLIENT: bool>(
  enc_buffer: &mut Vector<u8>,
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
) -> crate::Result<()> {
  enc_buffer.clear();
  match headers.trailers() {
    Trailers::None => {
      if IS_CLIENT {
        hpack_enc.encode(enc_buffer, hsreqh.iter(), headers.iter())?;
      } else {
        hpack_enc.encode(enc_buffer, hsresh.iter(), headers.iter())?;
      }
    }
    Trailers::Mixed => {
      let iter = headers.iter().filter(|el| !el.is_trailer);
      if IS_CLIENT {
        hpack_enc.encode(enc_buffer, hsreqh.iter(), iter)?;
      } else {
        hpack_enc.encode(enc_buffer, hsresh.iter(), iter)?;
      }
    }
    Trailers::Tail(idx) => {
      let iter = headers.iter().take(idx);
      if IS_CLIENT {
        hpack_enc.encode(enc_buffer, hsreqh.iter(), iter)?;
      } else {
        hpack_enc.encode(enc_buffer, hsresh.iter(), iter)?;
      }
    }
  }
  Ok(())
}

pub(crate) fn push_data(
  available_send: u32,
  data: &[u8],
  data_idx: &mut u32,
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  is_eos: bool,
  max_frame_len: u32,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<bool> {
  fn should_stop(data: &[u8], frame: &mut DataFrame, is_eos: bool, idx: u32) -> bool {
    if Usize::from_u32(idx).into_usize() >= data.len() {
      if is_eos {
        frame.set_eos();
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  if let Some(available_send_rest @ 1..=u32::MAX) = available_send.checked_sub(max_frame_len) {
    let (frame0_len @ 1..=u32::MAX, right0) = split_bytes(data, *data_idx, max_frame_len) else {
      return Ok(false);
    };
    let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
    let split_len = max_frame_len.min(available_send_rest);
    if let (frame1_len @ 1..=u32::MAX, _) = split_bytes(right0, 0, split_len) {
      let mut frame1 = DataFrame::new(frame1_len.into(), stream_id);
      let last_idx = data_idx.wrapping_add(frame0_len).wrapping_add(frame1_len);
      let should_stop = should_stop(data, &mut frame1, is_eos, last_idx);
      frames.push(frame_params!(data_idx, frame0, frame0_len, true))?;
      frames.push(frame_params!(data_idx, frame1, frame1_len, true))?;
      wp.withdrawn_send(Some(stream_id), frame0_len.wrapping_add(frame1_len).into())?;
      Ok(should_stop)
    } else {
      wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
      let last_idx = data_idx.wrapping_add(frame0_len);
      let should_stop = should_stop(data, &mut frame0, is_eos, last_idx);
      frames.push(frame_params!(data_idx, frame0, frame0_len, true))?;
      Ok(should_stop)
    }
  } else {
    let (frame0_len @ 1..=u32::MAX, _) = split_bytes(data, *data_idx, available_send) else {
      return Ok(false);
    };
    let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
    wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
    let last_idx = data_idx.wrapping_add(frame0_len);
    let should_stop = should_stop(data, &mut frame0, is_eos, last_idx);
    frames.push(frame_params!(data_idx, frame0, frame0_len, true))?;
    Ok(should_stop)
  }
}

pub(crate) fn push_headers<const IS_CLIENT: bool>(
  enc_buffer: &[u8],
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  hpack_idx: &mut u32,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_eos: bool,
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<bool> {
  let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
  let should_stop = if is_eos {
    frame0.set_eos();
    true
  } else {
    false
  };
  push_headers_or_trailers(
    &mut frame0,
    frames,
    hpack_idx,
    split_bytes(enc_buffer, 0, max_frame_len),
    max_frame_len,
    stream_id,
  )?;
  Ok(should_stop)
}

pub(crate) fn push_trailers(
  enc_buffer: &mut Vector<u8>,
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
  hpack_idx: &mut u32,
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<bool> {
  enc_buffer.clear();
  *hpack_idx = 0;
  encode_trailers(enc_buffer, headers, hpack_enc)?;
  let (left0 @ 1..=u32::MAX, right0) = split_bytes(enc_buffer, 0, max_frame_len) else {
    return Ok(false);
  };
  let mut frame0 = HeadersFrame::new(
    (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY),
    stream_id,
  );
  frame0.set_eos();
  push_headers_or_trailers(
    &mut frame0,
    frames,
    hpack_idx,
    (left0, right0),
    max_frame_len,
    stream_id,
  )?;
  Ok(true)
}

/// A message is composed by header frames, data frames and trailer frames.
///
/// 1. Header (1), Continuation (0 | 1)
/// 2. Data (0+)
/// 3. Trailer (0 | 1), Continuation (0 | 1)
///
/// * Control frames like Settings or `WindowUpdate` are out of scope.
/// * At most one continuation frame can be sent
pub(crate) async fn send_msg<HB, SW, const IS_CLIENT: bool>(
  data: &[u8],
  enc_buffer: &mut Vector<u8>,
  headers: &Headers,
  inner: &Http2Inner<HB, SW, IS_CLIENT>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  mut cb: impl FnMut(Http2DataPartsMut<'_, IS_CLIENT>),
) -> crate::Result<Http2SendStatus>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  let fut = async {
    let mut data_idx = 0;
    let mut frames = ArrayVectorU8::new();
    let mut hd_guard_pin = pin!(inner.hd.lock());
    let mut hpack_idx = 0;
    let hss = loop {
      let state = poll_fn(|cx| {
        if connection_state(&inner.is_conn_open).is_closed() {
          return Poll::Ready(crate::Result::Ok(SendMsgState::ClosedConnection));
        }
        let state = do_send_msg::<_, IS_CLIENT>(
          data,
          &mut data_idx,
          enc_buffer,
          &mut frames,
          &mut *lock_pin!(cx, inner.hd, hd_guard_pin),
          headers,
          &mut hpack_idx,
          (hsreqh, hsresh),
          stream_id,
          cx.waker(),
          &mut cb,
        )?;
        if let SendMsgState::NeedsMoreWindow = state {
          return Poll::Pending;
        };
        Poll::Ready(Ok(state))
      })
      .await?;
      let should_stop = match state {
        SendMsgState::ClosedConnection => break Http2SendStatus::ClosedConnection,
        SendMsgState::ClosedStream => break Http2SendStatus::ClosedStream,
        SendMsgState::Finished | SendMsgState::NeedsMoreWindow => break Http2SendStatus::Ok,
        SendMsgState::GeneratedData(should_stop) => should_stop,
        SendMsgState::GeneratedFastPath => true,
        SendMsgState::GeneratedHeaders(should_stop) => should_stop,
        SendMsgState::GeneratedHeadersAndData(should_stop) => should_stop,
        SendMsgState::GeneratedTrailers => true,
      };
      write_frames(
        (enc_buffer.as_ref(), data),
        &frames,
        &inner.is_conn_open,
        &mut inner.wd.lock().await.stream_writer,
      )
      .await?;
      if should_stop {
        break Http2SendStatus::Ok;
      }
      continue;
    };
    _trace!("Message has been sent");
    Ok(hss)
  };
  let rslt = fut.await;
  enc_buffer.clear();
  if let Err(err) = &rslt {
    process_higher_operation_err(err, inner).await;
  }
  rslt
}

pub(crate) async fn write_frames<SW>(
  (header, data): (&[u8], &[u8]),
  frames: &ArrayVectorU8<FrameParams, 4>,
  is_conn_open: &AtomicBool,
  stream_writer: &mut SW,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  fn get<'bytes>(
    (header, data): (&'bytes [u8], &'bytes [u8]),
    params: &FrameParams,
  ) -> &'bytes [u8] {
    let [begin, end] = params.range;
    let range = *Usize::from(begin)..*Usize::from(end);
    let opt = if params.is_data { data.get(range) } else { header.get(range) };
    opt.unwrap_or_default()
  }

  match frames.as_ref() {
    [a] => {
      let a_bytes = get((header, data), a);
      write_array([&a.header, a_bytes], is_conn_open, stream_writer).await?;
    }
    [a, b] => {
      let a_bytes = get((header, data), a);
      let b_bytes = get((header, data), b);
      write_array([&a.header, a_bytes, &b.header, b_bytes], is_conn_open, stream_writer).await?;
    }
    [a, b, c] => {
      let a_bytes = get((header, data), a);
      let b_bytes = get((header, data), b);
      let c_bytes = get((header, data), c);
      write_array(
        [&a.header, a_bytes, &b.header, b_bytes, &c.header, c_bytes],
        is_conn_open,
        stream_writer,
      )
      .await?;
    }
    [a, b, c, d] => {
      let a_bytes = get((header, data), a);
      let b_bytes = get((header, data), b);
      let c_bytes = get((header, data), c);
      let d_bytes = get((header, data), d);
      write_array(
        [&a.header, a_bytes, &b.header, b_bytes, &c.header, c_bytes, &d.header, d_bytes],
        is_conn_open,
        stream_writer,
      )
      .await?;
    }
    _ => {}
  }
  Ok(())
}

fn change_final_stream_state<const IS_CLIENT: bool>(stream_state: &mut StreamState) {
  *stream_state = if IS_CLIENT { StreamState::HalfClosedLocal } else { StreamState::Closed };
}

fn change_initial_stream_state<const IS_CLIENT: bool>(stream_state: &mut StreamState) {
  if IS_CLIENT {
    *stream_state = StreamState::Open;
  }
}

fn data_frame_len(bytes_len: usize) -> u32 {
  u32::try_from(bytes_len).unwrap_or_default()
}

fn do_send_msg<HB, const IS_CLIENT: bool>(
  data: &[u8],
  data_idx: &mut u32,
  enc_buffer: &mut Vector<u8>,
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  hd: &mut Http2Data<HB, IS_CLIENT>,
  headers: &Headers,
  hpack_idx: &mut u32,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  waker: &Waker,
  cb: &mut impl FnMut(Http2DataPartsMut<'_, IS_CLIENT>),
) -> crate::Result<SendMsgState>
where
  HB: LeaseMut<Http2Buffer>,
{
  frames.clear();
  let hdpm = hd.parts_mut();
  let scrp = scrp_mut(&mut hdpm.hb.scrps, stream_id)?;
  if !scrp.is_stream_open {
    return Ok(SendMsgState::ClosedStream);
  }
  if !scrp.stream_state.can_send::<IS_CLIENT>() {
    return Err(protocol_err(Http2Error::InvalidSendStreamState));
  }
  let mut wp = WindowsPair::new(hdpm.windows, &mut scrp.windows);

  let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
    if enc_buffer.is_empty() {
      encode_headers::<IS_CLIENT>(enc_buffer, headers, &mut hdpm.hb.hpack_enc, (hsreqh, hsresh))?;
      let should_stop = push_headers::<IS_CLIENT>(
        enc_buffer,
        frames,
        hpack_idx,
        (hsreqh, hsresh),
        is_eos(data, *data_idx, headers),
        hdpm.hps.max_frame_len,
        stream_id,
      )?;
      if should_stop {
        final_generic_calls!(cb, hdpm, scrp);
      } else {
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        scrp.waker.clone_from(waker);
      }
      return Ok(SendMsgState::GeneratedHeaders(should_stop));
    }
    scrp.waker.clone_from(waker);
    return Ok(SendMsgState::NeedsMoreWindow);
  };

  if enc_buffer.is_empty() {
    push_fast_path::<IS_CLIENT>(
      available_send,
      data,
      data_idx,
      enc_buffer,
      frames,
      headers,
      &mut hdpm.hb.hpack_enc,
      hpack_idx,
      (hsreqh, hsresh),
      hdpm.hps.max_frame_len,
      stream_id,
      &mut wp,
    )?;
    if !frames.is_empty() {
      final_generic_calls!(cb, hdpm, scrp);
      return Ok(SendMsgState::GeneratedFastPath);
    }
    let should_stop = push_headers::<IS_CLIENT>(
      enc_buffer,
      frames,
      hpack_idx,
      (hsreqh, hsresh),
      is_eos(data, *data_idx, headers),
      hdpm.hps.max_frame_len,
      stream_id,
    )?;
    if should_stop {
      final_generic_calls!(cb, hdpm, scrp);
      return Ok(SendMsgState::GeneratedHeaders(true));
    } else {
      change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
      let should_stop = push_data(
        available_send,
        data,
        data_idx,
        frames,
        !headers.trailers().has_any(),
        hdpm.hps.max_frame_len,
        stream_id,
        &mut wp,
      )?;
      if should_stop {
        final_generic_calls!(cb, hdpm, scrp);
      } else {
        scrp.waker.clone_from(waker);
        if wp.available_send() > 0 {
          waker.wake_by_ref();
        }
      }
      return Ok(SendMsgState::GeneratedHeadersAndData(should_stop));
    }
  }

  if *Usize::from(*data_idx) < data.len() {
    let should_stop = push_data(
      available_send,
      data,
      data_idx,
      frames,
      !headers.trailers().has_any(),
      hdpm.hps.max_frame_len,
      stream_id,
      &mut wp,
    )?;
    if should_stop {
      final_generic_calls!(cb, hdpm, scrp);
    } else {
      scrp.waker.clone_from(waker);
      if wp.available_send() > 0 {
        waker.wake_by_ref();
      }
    }
    return Ok(SendMsgState::GeneratedData(should_stop));
  }

  if headers.trailers().has_any() {
    let is_not_empty = push_trailers(
      enc_buffer,
      frames,
      headers,
      &mut hdpm.hb.hpack_enc,
      hpack_idx,
      hdpm.hps.max_frame_len,
      stream_id,
    )?;
    if is_not_empty {
      final_generic_calls!(cb, hdpm, scrp);
      return Ok(SendMsgState::GeneratedTrailers);
    }
  }

  Ok(SendMsgState::Finished)
}

fn encode_trailers(
  enc_buffer: &mut Vector<u8>,
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
) -> crate::Result<()> {
  match headers.trailers() {
    Trailers::None => {}
    Trailers::Mixed => {
      hpack_enc.encode(enc_buffer, [], headers.iter().filter(|el| el.is_trailer))?;
    }
    Trailers::Tail(idx) => {
      hpack_enc.encode(enc_buffer, [], headers.iter().skip(idx))?;
    }
  }
  Ok(())
}

fn has_delimited_bytes(data_bytes: &[u8], available_len: u32) -> Option<U31> {
  if !data_bytes.is_empty() && data_bytes.len() <= *Usize::from(available_len) {
    return Some(U31::from_u32(u32::try_from(data_bytes.len()).ok()?));
  }
  None
}

fn is_eos(data: &[u8], data_idx: u32, headers: &Headers) -> bool {
  *Usize::from(data_idx) >= data.len() && !headers.trailers().has_any()
}

fn push_fast_path<const IS_CLIENT: bool>(
  available_send: u32,
  data: &[u8],
  data_idx: &mut u32,
  enc_buffer: &mut Vector<u8>,
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
  hpack_idx: &mut u32,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  max_frame_len: u32,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<()> {
  encode_headers::<IS_CLIENT>(enc_buffer, headers, hpack_enc, (hsreqh, hsresh))?;
  let Some(data_len) = has_delimited_bytes(data, available_send.min(max_frame_len)) else {
    return Ok(());
  };
  if headers.trailers().has_any() {
    let idx = enc_buffer.len();
    encode_trailers(enc_buffer, headers, hpack_enc)?;
    let Some((headers_bytes, trailers_bytes)) = enc_buffer.split_at_checked(idx) else {
      enc_buffer.truncate(idx);
      return Ok(());
    };
    let Some(_) = has_delimited_bytes(headers_bytes, max_frame_len) else {
      enc_buffer.truncate(idx);
      return Ok(());
    };
    let Some(_) = has_delimited_bytes(trailers_bytes, max_frame_len) else {
      enc_buffer.truncate(idx);
      return Ok(());
    };
    let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
    let frame1 = DataFrame::new(data_len, stream_id);
    let mut frame2 = HeadersFrame::new(
      (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY),
      stream_id,
    );
    frame0.set_eoh();
    frame2.set_eoh();
    frame2.set_eos();
    frames.push(frame_params!(hpack_idx, frame0, data_frame_len(headers_bytes.len()), false))?;
    frames.push(frame_params!(data_idx, frame1, data_len.u32(), true))?;
    frames.push(frame_params!(hpack_idx, frame2, data_frame_len(trailers_bytes.len()), false))?;
  } else {
    let Some(_) = has_delimited_bytes(enc_buffer, max_frame_len) else {
      return Ok(());
    };
    let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
    let mut frame1 = DataFrame::new(data_len, stream_id);
    frame0.set_eoh();
    frame1.set_eos();
    frames.push(frame_params!(hpack_idx, frame0, data_frame_len(enc_buffer.len()), false))?;
    frames.push(frame_params!(data_idx, frame1, data_len.u32(), true))?;
  }
  wp.withdrawn_send(Some(stream_id), data_len)?;
  Ok(())
}

fn push_headers_or_trailers(
  frame0: &mut HeadersFrame<'_>,
  frames: &mut ArrayVectorU8<FrameParams, 4>,
  hpack_idx: &mut u32,
  (left0_len, right0): (u32, &[u8]),
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<()> {
  if let (left1_len @ 1..=u32::MAX, right1) = split_bytes(right0, 0, max_frame_len) {
    let mut frame1 = ContinuationFrame::new(stream_id);
    if !right1.is_empty() {
      return Err(protocol_err(Http2Error::HeadersOverflow));
    }
    frame1.set_eoh();
    frames.push(frame_params!(hpack_idx, frame0, left0_len, false))?;
    frames.push(frame_params!(hpack_idx, frame1, left1_len, false))?;
  } else {
    frame0.set_eoh();
    frames.push(frame_params!(hpack_idx, frame0, left0_len, false))?;
  }
  Ok(())
}

fn split_bytes(bytes: &[u8], begin: u32, len: u32) -> (u32, &[u8]) {
  if let Some(rest @ [_not_empty, ..]) = bytes.get(*Usize::from(begin)..) {
    if let Some((lhs, rhs)) = rest.split_at_checked(*Usize::from(len)) {
      (data_frame_len(lhs.len()), rhs)
    } else {
      (data_frame_len(rest.len()), &[])
    }
  } else {
    (0, &[])
  }
}

#[derive(Clone, Copy, Debug)]
enum SendMsgState {
  ClosedConnection,
  ClosedStream,
  Finished,
  GeneratedData(bool),
  GeneratedFastPath,
  GeneratedHeaders(bool),
  GeneratedHeadersAndData(bool),
  GeneratedTrailers,
  NeedsMoreWindow,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FrameParams {
  header: [u8; 9],
  is_data: bool,
  range: [u32; 2],
}
