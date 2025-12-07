//! A message is composed by header frames, data frames and trailer frames.
//!
//! 1. Header (1), Continuation (0 | 1)
//! 2. Data (0+)
//! 3. Trailer (0 | 1), Continuation (0 | 1)
//!
//! * Control frames like Settings or `WindowUpdate` are out of scope.
//! * At most one continuation frame can be sent

macro_rules! final_generic_calls {
  ($cb:ident, $hd_guard:ident, $hdpm:ident, $scrp:ident) => {{
    change_final_stream_state::<IS_CLIENT>(&mut $scrp.stream_state);
    $cb($hdpm);
    drop($hd_guard);
  }};
}

macro_rules! init {
  ($frame_len:expr, $frame:expr) => {{
    let mut buffer = [0; 9];
    let [a, b, c, d, e, f, g, h, i] = &mut buffer;
    let [_not_empty, j, k, l] = $frame_len.to_be_bytes();
    let [_not_empty, _, _, m, n, o, p, q, r] = $frame.bytes();
    *a = j;
    *b = k;
    *c = l;
    *d = m;
    *e = n;
    *f = o;
    *g = p;
    *h = q;
    *i = r;
    buffer
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
    writer_data::WriterData,
  },
  misc::{LeaseMut, Usize},
  stream::StreamWriter,
  sync::{AsyncMutexGuard, AtomicBool},
};
use core::{
  future::poll_fn,
  pin::pin,
  task::{Poll, Waker, ready},
};

pub(crate) fn encode_headers<const IS_CLIENT: bool>(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
) -> crate::Result<()> {
  hpack_enc_buffer.clear();
  match headers.trailers() {
    Trailers::None => {
      if IS_CLIENT {
        hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), headers.iter())?;
      } else {
        hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), headers.iter())?;
      }
    }
    Trailers::Mixed => {
      let iter = headers.iter().filter(|el| !el.is_trailer);
      if IS_CLIENT {
        hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), iter)?;
      } else {
        hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), iter)?;
      }
    }
    Trailers::Tail(idx) => {
      let iter = headers.iter().take(idx);
      if IS_CLIENT {
        hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), iter)?;
      } else {
        hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), iter)?;
      }
    }
  }
  Ok(())
}

/// Tries to send up two data frames in a single round trip. If exhausted, returns `true`.
pub(crate) fn gen_standalone_data<'data, 'rslt>(
  available_send: u32,
  data: &mut &'data [u8],
  force_eos: bool,
  has_data: &mut bool,
  has_trailers: bool,
  max_frame_len: u32,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<(ArrayVectorU8<([u8; 9], &'rslt [u8]), 2>, bool)>
where
  'data: 'rslt,
{
  const fn should_stop(
    data: &[u8],
    frame: &mut DataFrame,
    has_data: &mut bool,
    has_trailers: bool,
  ) -> bool {
    if data.is_empty() {
      *has_data = true;
      if has_trailers {
        false
      } else {
        frame.set_eos();
        true
      }
    } else {
      false
    }
  }
  let mut frames = ArrayVectorU8::new();
  if let Some(available_send_rest @ 1..=u32::MAX) = available_send.checked_sub(max_frame_len) {
    let (left0 @ [_not_empty, ..], right0) = split_frame_bytes(data, max_frame_len) else {
      *has_data = true;
      return Ok((frames, false));
    };
    let frame0_len = data_frame_len(left0.len());
    let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
    let split_len = max_frame_len.min(available_send_rest);
    if let (left1 @ [_not_empty, ..], right1) = split_frame_bytes(right0, split_len) {
      let frame1_len = data_frame_len(left1.len());
      let mut frame1 = DataFrame::new(frame1_len.into(), stream_id);
      let should_stop = should_stop(right1, &mut frame1, has_data, has_trailers);
      if force_eos {
        frame1.set_eos();
      }
      *data = right1;
      let _rslt = frames.push((init!(frame0_len, frame0), left0));
      let _rslt = frames.push((init!(frame1_len, frame1), left1));
      wp.withdrawn_send(Some(stream_id), frame0_len.wrapping_add(frame1_len).into())?;
      Ok((frames, should_stop))
    } else {
      let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
      if force_eos {
        frame0.set_eos();
      }
      wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
      *data = right0;
      let _rslt = frames.push((init!(frame0_len, frame0), left0));
      Ok((frames, should_stop))
    }
  } else {
    let tuple = split_frame_bytes(data, available_send);
    let (left0 @ [_not_empty, ..], right0) = tuple else {
      *has_data = true;
      return Ok((frames, false));
    };
    let frame0_len = data_frame_len(left0.len());
    let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
    let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
    if force_eos {
      frame0.set_eos();
    }
    wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
    *data = right0;
    let _rslt = frames.push((init!(frame0_len, frame0), left0));
    Ok((frames, should_stop))
  }
}

// Tries to send all initial headers
pub(crate) fn gen_standalone_headers<'data, 'rslt, const IS_CLIENT: bool>(
  hpack_enc_buffer: &'data [u8],
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_eos: bool,
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<(ArrayVectorU8<([u8; 9], &'rslt [u8]), 2>, bool)>
where
  'data: 'rslt,
{
  let (left0, right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len);
  let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
  let should_stop = if is_eos {
    frame0.set_eos();
    true
  } else {
    false
  };
  Ok((
    gen_headers_or_trailers(&mut frame0, (left0, right0), max_frame_len, stream_id)?,
    should_stop,
  ))
}

/// Tries to send all trailer headers
pub(crate) fn gen_standalone_trailers<'data, 'rslt>(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &'data mut Vector<u8>),
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<ArrayVectorU8<([u8; 9], &'rslt [u8]), 2>>
where
  'data: 'rslt,
{
  hpack_enc_buffer.clear();
  encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
  let (left0 @ [_not_empty, ..], right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len)
  else {
    return Ok(ArrayVectorU8::new());
  };
  let mut frame0 = HeadersFrame::new(
    (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY),
    stream_id,
  );
  frame0.set_eos();
  gen_headers_or_trailers(&mut frame0, (left0, right0), max_frame_len, stream_id)
}

pub(crate) async fn send_msg<HB, SW, const IS_CLIENT: bool>(
  mut data_bytes: &[u8],
  inner: &Http2Inner<HB, SW, IS_CLIENT>,
  headers: &Headers,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  mut cb: impl FnMut(Http2DataPartsMut<'_, IS_CLIENT>),
) -> crate::Result<Http2SendStatus>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  let (mut has_headers, mut has_data) = (false, false);

  let rslt = {
    let mut hd_guard_pin = pin!(inner.hd.lock());
    let mut wd_guard_pin = pin!(inner.wd.lock());
    poll_fn(move |cx| {
      if connection_state(&inner.is_conn_open).is_closed() {
        return Poll::Ready(Ok(Http2SendStatus::ClosedConnection));
      }
      let hd_guard = lock_pin!(cx, inner.hd, hd_guard_pin);
      let mut wd_guard = lock_pin!(cx, inner.wd, wd_guard_pin);
      let fut = do_send_msg::<_, _, IS_CLIENT>(
        &mut data_bytes,
        (&mut has_headers, &mut has_data),
        headers,
        hd_guard,
        (hsreqh, hsresh),
        &inner.is_conn_open,
        stream_id,
        cx.waker(),
        &mut *wd_guard,
        &mut cb,
      );
      let rslt = ready!(pin!(fut).poll(cx));
      if let Some(is_fully_sent) = rslt? {
        if is_fully_sent {
          if IS_CLIENT {
            _trace!("Request has been sent");
          } else {
            _trace!("Response has been sent");
          };
          Poll::Ready(Ok(Http2SendStatus::Ok))
        } else {
          Poll::Pending
        }
      } else {
        Poll::Ready(Ok(Http2SendStatus::ClosedStream))
      }
    })
    .await
  };
  if let Err(err) = &rslt {
    process_higher_operation_err(err, inner).await;
  }
  rslt
}

pub(crate) async fn write_data_frames<SW>(
  frames: ArrayVectorU8<([u8; 9], &[u8]), 2>,
  is_conn_open: &AtomicBool,
  stream_writer: &mut SW,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  match frames.as_ref() {
    [(a, b)] => {
      write_array([a, b], is_conn_open, stream_writer).await?;
    }
    [(a, b), (c, d)] => {
      write_array([a, b, c, d], is_conn_open, stream_writer).await?;
    }
    _ => {}
  }
  Ok(())
}

pub(crate) async fn write_data_and_header_frames<SW>(
  data_frames: ArrayVectorU8<([u8; 9], &[u8]), 2>,
  header_frames: ArrayVectorU8<([u8; 9], &[u8]), 2>,
  is_conn_open: &AtomicBool,
  stream_writer: &mut SW,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  match (header_frames.as_ref(), data_frames.as_ref()) {
    ([(a, b)], [(c, d)]) => {
      write_array([a, b, c, d], is_conn_open, stream_writer).await?;
    }
    ([(a, b), (c, d)], [(e, f)]) | ([(a, b)], [(c, d), (e, f)]) => {
      write_array([a, b, c, d, e, f], is_conn_open, stream_writer).await?;
    }
    ([(a, b), (c, d)], [(e, f), (g, h)]) => {
      write_array([a, b, c, d, e, f, g, h], is_conn_open, stream_writer).await?;
    }
    _ => {}
  }
  Ok(())
}

pub(crate) async fn write_header_frames<SW>(
  frames: ArrayVectorU8<([u8; 9], &[u8]), 2>,
  is_conn_open: &AtomicBool,
  stream_writer: &mut SW,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  match frames.as_ref() {
    [(a, b)] => {
      write_array([a, b], is_conn_open, stream_writer).await?;
    }
    [(a, b), (c, d)] => {
      write_array([a, b, c, d], is_conn_open, stream_writer).await?;
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

/// Returns `true` if the message was fully sent.
///
/// Tries to at least send initial headers when the windows size does not allow sending data frames.
async fn do_send_msg<HB, SW, const IS_CLIENT: bool>(
  data_bytes: &mut &[u8],
  (has_headers, has_data): (&mut bool, &mut bool),
  headers: &Headers,
  mut hd_guard: AsyncMutexGuard<'_, Http2Data<HB, IS_CLIENT>>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: &AtomicBool,
  stream_id: U31,
  waker: &Waker,
  wd: &mut WriterData<SW>,
  cb: &mut impl FnMut(Http2DataPartsMut<'_, IS_CLIENT>),
) -> crate::Result<Option<bool>>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  let hdpm = hd_guard.parts_mut();
  let scrp = scrp_mut(&mut hdpm.hb.scrp, stream_id)?;
  if !scrp.is_stream_open {
    return Ok(None);
  }
  if !scrp.stream_state.can_send::<IS_CLIENT>() {
    return Err(protocol_err(Http2Error::InvalidSendStreamState));
  }
  hdpm.hb.hpack_enc_buffer.clear();
  let max_frame_len = hdpm.hps.max_frame_len;
  let mut wp = WindowsPair::new(hdpm.windows, &mut scrp.windows);

  let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
    if !*has_headers {
      let hpack = (&mut hdpm.hb.hpack_enc, &mut wd.hpack_enc_buffer);
      encode_headers::<IS_CLIENT>(headers, hpack, (hsreqh, hsresh))?;
      let (frames, should_stop) = gen_standalone_headers::<IS_CLIENT>(
        &mut wd.hpack_enc_buffer,
        (hsreqh, hsresh),
        data_bytes.is_empty() && !headers.trailers().has_any(),
        max_frame_len,
        stream_id,
      )?;
      if should_stop {
        final_generic_calls!(cb, hd_guard, hdpm, scrp);
      } else {
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        *has_headers = true;
        scrp.waker.clone_from(waker);
        drop(hd_guard);
      }
      write_header_frames(frames, is_conn_open, &mut wd.stream_writer).await?;
      return Ok(Some(should_stop));
    }
    scrp.waker.clone_from(waker);
    return Ok(Some(false));
  };
  if !*has_headers {
    {
      let frames = gen_fast_path::<IS_CLIENT>(
        available_send,
        data_bytes,
        headers,
        (&mut hdpm.hb.hpack_enc, &mut wd.hpack_enc_buffer),
        (hsreqh, hsresh),
        max_frame_len,
        stream_id,
        &mut wp,
      )?;
      match frames.as_ref() {
        [(a, b), (c, d)] => {
          final_generic_calls!(cb, hd_guard, hdpm, scrp);
          write_array([a, b, c, d], is_conn_open, &mut wd.stream_writer).await?;
          return Ok(Some(true));
        }
        [(a, b), (c, d), (e, f)] => {
          final_generic_calls!(cb, hd_guard, hdpm, scrp);
          write_array([a, b, c, d, e, f], is_conn_open, &mut wd.stream_writer).await?;
          return Ok(Some(true));
        }
        _ => {}
      }
    }
    let (header_frames, should_stop) = gen_standalone_headers::<IS_CLIENT>(
      &mut wd.hpack_enc_buffer,
      (hsreqh, hsresh),
      data_bytes.is_empty() && !headers.trailers().has_any(),
      max_frame_len,
      stream_id,
    )?;
    if should_stop {
      final_generic_calls!(cb, hd_guard, hdpm, scrp);
      *has_headers = true;
      write_header_frames(header_frames, is_conn_open, &mut wd.stream_writer).await?;
      return Ok(Some(true));
    } else {
      change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
      let (data_frames, should_stop) = gen_standalone_data(
        available_send,
        data_bytes,
        false,
        has_data,
        headers.trailers().has_any(),
        max_frame_len,
        stream_id,
        &mut wp,
      )?;
      *has_headers = true;
      if should_stop {
        final_generic_calls!(cb, hd_guard, hdpm, scrp);
      } else {
        scrp.waker.clone_from(waker);
        if wp.available_send() > 0 {
          waker.wake_by_ref();
        }
        drop(hd_guard);
      }
      write_data_and_header_frames(data_frames, header_frames, is_conn_open, &mut wd.stream_writer)
        .await?;
      return Ok(Some(should_stop));
    }
  }

  if !*has_data {
    let (frames, should_stop) = gen_standalone_data(
      available_send,
      data_bytes,
      false,
      has_data,
      headers.trailers().has_any(),
      max_frame_len,
      stream_id,
      &mut wp,
    )?;
    if should_stop {
      final_generic_calls!(cb, hd_guard, hdpm, scrp);
    } else {
      scrp.waker.clone_from(waker);
      if wp.available_send() > 0 {
        waker.wake_by_ref();
      }
      drop(hd_guard);
    }
    write_data_frames(frames, is_conn_open, &mut wd.stream_writer).await?;
    return Ok(Some(should_stop));
  }

  if headers.trailers().has_any() {
    let hpack = (&mut hdpm.hb.hpack_enc, &mut wd.hpack_enc_buffer);
    let frames = gen_standalone_trailers(headers, hpack, max_frame_len, stream_id)?;
    final_generic_calls!(cb, hd_guard, hdpm, scrp);
    write_header_frames(frames, is_conn_open, &mut wd.stream_writer).await?;
  }
  Ok(Some(true))
}

fn encode_trailers(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
) -> crate::Result<()> {
  match headers.trailers() {
    Trailers::None => {
      hpack_enc.encode(hpack_enc_buffer, [], headers.iter())?;
    }
    Trailers::Mixed => {
      hpack_enc.encode(hpack_enc_buffer, [], headers.iter().filter(|el| el.is_trailer))?;
    }
    Trailers::Tail(idx) => {
      hpack_enc.encode(hpack_enc_buffer, [], headers.iter().skip(idx))?;
    }
  }
  Ok(())
}

// Tries to send everything in a single round trip. If not possible, will at least send headers.
fn gen_fast_path<'data, 'headers, 'heb, 'rslt, const IS_CLIENT: bool>(
  available_send: u32,
  data_bytes: &'data [u8],
  headers: &'headers Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &'heb mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  max_frame_len: u32,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<ArrayVectorU8<([u8; 9], &'rslt [u8]), 3>>
where
  'data: 'rslt,
  'heb: 'rslt,
  'headers: 'rslt,
{
  let mut frames = ArrayVectorU8::new();
  encode_headers::<IS_CLIENT>(headers, (hpack_enc, hpack_enc_buffer), (hsreqh, hsresh))?;
  let Some(data_len) = has_delimited_bytes(data_bytes, available_send.min(max_frame_len)) else {
    return Ok(frames);
  };
  if headers.trailers().has_any() {
    let idx = hpack_enc_buffer.len();
    encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
    let Some((headers_bytes, trailers_bytes)) = hpack_enc_buffer.split_at_checked(idx) else {
      return Ok(frames);
    };
    let Some(_) = has_delimited_bytes(headers_bytes, max_frame_len) else {
      return Ok(frames);
    };
    let Some(_) = has_delimited_bytes(trailers_bytes, max_frame_len) else {
      return Ok(frames);
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
    frames.push((init!(data_frame_len(headers_bytes.len()), frame0), headers_bytes))?;
    frames.push((init!(data_frame_len(data_bytes.len()), frame1), data_bytes))?;
    frames.push((init!(data_frame_len(trailers_bytes.len()), frame2), trailers_bytes))?;
  } else {
    let Some(_) = has_delimited_bytes(hpack_enc_buffer, max_frame_len) else {
      return Ok(frames);
    };
    let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
    let mut frame1 = DataFrame::new(data_len, stream_id);
    frame0.set_eoh();
    frame1.set_eos();
    frames.push((init!(data_frame_len(hpack_enc_buffer.len()), frame0), hpack_enc_buffer))?;
    frames.push((init!(data_frame_len(data_bytes.len()), frame1), data_bytes))?;
  }
  wp.withdrawn_send(Some(stream_id), data_len)?;
  Ok(frames)
}

fn gen_headers_or_trailers<'data, 'rslt>(
  frame0: &mut HeadersFrame<'_>,
  (left0, right0): (&'data [u8], &'data [u8]),
  max_frame_len: u32,
  stream_id: U31,
) -> crate::Result<ArrayVectorU8<([u8; 9], &'rslt [u8]), 2>>
where
  'data: 'rslt,
{
  let mut frames = ArrayVectorU8::new();
  if let (left1 @ [_not_empty, ..], right1) = split_frame_bytes(right0, max_frame_len) {
    let mut frame1 = ContinuationFrame::new(stream_id);
    if !right1.is_empty() {
      return Err(protocol_err(Http2Error::HeadersOverflow));
    }
    frame1.set_eoh();
    frames.push((init!(data_frame_len(left0.len()), frame0), left0))?;
    frames.push((init!(data_frame_len(left1.len()), frame1), left1))?;
  } else {
    frame0.set_eoh();
    frames.push((init!(data_frame_len(left0.len()), frame0), left0))?;
  }
  Ok(frames)
}

fn has_delimited_bytes(data_bytes: &[u8], available_len: u32) -> Option<U31> {
  if !data_bytes.is_empty() && data_bytes.len() <= *Usize::from(available_len) {
    return Some(U31::from_u32(u32::try_from(data_bytes.len()).ok()?));
  }
  None
}

fn split_frame_bytes(bytes: &[u8], len: u32) -> (&[u8], &[u8]) {
  match bytes.split_at_checked(*Usize::from(len)) {
    Some(elem) => elem,
    None => (bytes, &[]),
  }
}
