//! A message is composed by header frames, data frames and trailer frames.
//!
//! Control frames like Settings or WindowUpdate are out of scope.

macro_rules! init {
  ($bytes:expr, $frame:expr) => {{
    let mut buffer = [0; 9];
    adjust_frame_init($bytes, $frame.bytes(), &mut buffer);
    buffer
  }};
}

macro_rules! loop_checks {
  (($scrp:ident, $wp:ident), $hdpm:expr, $stream_id:expr) => {{
    let Some(local_scrp) = $hdpm.hb.scrp.get_mut(&$stream_id) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::BadLocalFlow));
    };
    if !local_scrp.stream_state.can_send_stream::<IS_CLIENT>() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidSendStreamState));
    }
    $scrp = local_scrp;
    let local_wp = WindowsPair::new($hdpm.windows, &mut $scrp.windows);
    if local_wp.is_invalid_send() {
      continue;
    }
    $wp = local_wp;
  }};
}

use crate::{
  http::{Header, Headers},
  http2::{
    http2_data::Http2DataPartsMut, misc::write_array, window::WindowsPair, ContinuationFrame,
    DataFrame, HeadersFrame, HpackEncoder, HpackHeaderBasic, HpackStaticRequestHeaders,
    HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2Error, StreamBuffer, StreamState, U31,
  },
  misc::{ByteVector, LeaseMut, Lock, RefCounter, Stream, Usize},
};
use tokio::sync::MutexGuard;

#[inline]
pub(crate) async fn send_msg<HB, HD, S, SB, const IS_CLIENT: bool>(
  mut data_bytes: &[u8],
  hd: &HD,
  headers: &Headers,
  hpack_enc_buffer: &mut ByteVector,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  cb: impl FnMut(Http2DataPartsMut<'_, S, SB>),
) -> crate::Result<()>
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
  let (max_frame_len, mut headers_bytes, mut trailers_bytes) = {
    let mut guard = hd.lock().await;
    let hdpm = guard.parts_mut();
    let (headers_bytes, trailers_bytes) = if IS_CLIENT {
      encode_headers(headers, &mut hdpm.hb.hpack_enc, hpack_enc_buffer, hsreqh.iter())?
    } else {
      encode_headers(headers, &mut hdpm.hb.hpack_enc, hpack_enc_buffer, hsresh.iter())?
    };
    (*Usize::from(hdpm.hps.max_frame_len), headers_bytes, trailers_bytes)
  };
  fast_path(
    (&mut headers_bytes, &mut data_bytes, &mut trailers_bytes),
    hd,
    (hsreqh, hsresh),
    max_frame_len,
    stream_id,
  )
  .await?;
  if headers_bytes.is_empty() && data_bytes.is_empty() && trailers_bytes.is_empty() {
    return Ok(());
  }
  slow_path(
    (&mut headers_bytes, &mut data_bytes, &mut trailers_bytes),
    hd,
    max_frame_len,
    stream_id,
    cb,
  )
  .await
}

#[inline]
fn adjust_frame_init(content: &[u8], frame_init: [u8; 9], frame_init_buffer: &mut [u8; 9]) {
  let [a, b, c, d, e, f, g, h, i] = frame_init_buffer;
  let [_, j, k, l] = u32::try_from(content.len()).unwrap_or_default().to_be_bytes();
  let [_, _, _, m, n, o, p, q, r] = frame_init;
  *a = j;
  *b = k;
  *c = l;
  *d = m;
  *e = n;
  *f = o;
  *g = p;
  *h = q;
  *i = r;
}

#[inline]
fn change_final_stream_state<const IS_CLIENT: bool>(stream_state: &mut StreamState) {
  *stream_state = if IS_CLIENT { StreamState::HalfClosedLocal } else { StreamState::Closed };
}

#[inline]
fn change_initial_stream_state<const IS_CLIENT: bool>(stream_state: &mut StreamState) {
  if IS_CLIENT {
    *stream_state = StreamState::Open
  }
}

#[inline]
fn data_frame_len(bytes: &[u8]) -> U31 {
  U31::from_u32(u32::try_from(bytes.len()).unwrap_or_default())
}

#[inline]
fn encode_headers<'buffer, 'pseudo>(
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
  hpack_enc_buffer: &'buffer mut ByteVector,
  pseudo_headers: impl Iterator<Item = (HpackHeaderBasic, &'pseudo [u8])>,
) -> crate::Result<(&'buffer [u8], &'buffer [u8])> {
  hpack_enc_buffer.clear();
  if headers.has_trailers() {
    let filter_cb = |header: &Header<'_>| !header.is_trailer;
    hpack_enc.encode(hpack_enc_buffer, pseudo_headers, headers.iter().filter(filter_cb))?;
    let before_trailers = hpack_enc_buffer.len();
    hpack_enc.encode(
      hpack_enc_buffer,
      [].into_iter(),
      headers.iter().filter(|el| el.is_trailer),
    )?;
    Ok(hpack_enc_buffer.split_at(before_trailers))
  } else {
    hpack_enc.encode(hpack_enc_buffer, pseudo_headers, headers.iter())?;
    Ok((&*hpack_enc_buffer, &[]))
  }
}

#[inline]
pub(crate) async fn fast_path<HB, HD, S, SB, const IS_CLIENT: bool>(
  (headers, data, trailers): (&mut &[u8], &mut &[u8], &mut &[u8]),
  hd: &HD,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  max_frame_len: usize,
  stream_id: U31,
) -> crate::Result<()>
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
  let second_idx = max_frame_len.wrapping_mul(2);
  let first_opt = headers.get(..max_frame_len);
  let second_opt = headers.get(max_frame_len..second_idx);
  let rest_opt = headers.get(second_idx..);
  if let (Some(first), Some(second), Some(rest)) = (first_opt, second_opt, rest_opt) {
    process_receipt_loop!(hd, |guard| {
      let hdpm = guard.parts_mut();
      let (scrp, _wp);
      loop_checks!((scrp, _wp), hdpm, stream_id);
      let mut hf = HeadersFrame::new((hsreqh, hsresh), stream_id);
      if data.is_empty() {
        hf.set_eos();
      }
      if rest.is_empty() {
        hf.set_eoh();
      }
      let headers_init = init!(first, hf);
      let continuation_init = init!(second, ContinuationFrame::new(stream_id));
      write_array(
        [&headers_init, first, &continuation_init, second],
        *hdpm.is_conn_open,
        hdpm.stream,
      )
      .await?;
      change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
      *headers = rest;
      break;
    });
  }

  let (first_headers, headers_rest) = split_frame_bytes(headers, max_frame_len);
  match (data.is_empty(), trailers.is_empty()) {
    (false, true) => {
      let (first_data, data_rest) = split_frame_bytes(data, max_frame_len);
      process_receipt_loop!(hd, |guard| {
        let hdpm = guard.parts_mut();
        let (scrp, mut wp);
        loop_checks!((scrp, wp), hdpm, stream_id);
        let mut hf = HeadersFrame::new((hsreqh, hsresh), stream_id);
        if headers_rest.is_empty() {
          hf.set_eoh();
        }
        let headers_init = init!(first_headers, hf);
        let data_frame_len = data_frame_len(first_data);
        let mut data_frame = DataFrame::new(data_frame_len, stream_id);
        if data_rest.is_empty() {
          data_frame.set_eos();
        }
        let data_init = init!(first_data, data_frame);
        if !wp.manage_send(data_frame_len) {
          continue;
        }
        write_array(
          [&headers_init, first_headers, &data_init, first_data],
          *hdpm.is_conn_open,
          hdpm.stream,
        )
        .await?;
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        *data = data_rest;
        break;
      });
    }
    (false, false) => {
      let (first_data, data_rest) = split_frame_bytes(data, max_frame_len);
      let (first_trailers, trailers_rest) = split_frame_bytes(trailers, max_frame_len);
      process_receipt_loop!(hd, |guard| {
        let hdpm = guard.parts_mut();
        let (scrp, mut wp);
        loop_checks!((scrp, wp), hdpm, stream_id);
        let mut hf = HeadersFrame::new((hsreqh, hsresh), stream_id);
        if headers_rest.is_empty() {
          hf.set_eoh();
        }
        let headers_init = init!(first_headers, hf);
        let data_frame_len = data_frame_len(first_data);
        let data_frame = DataFrame::new(data_frame_len, stream_id);
        let data_init = init!(first_data, data_frame);
        if !wp.manage_send(data_frame_len) {
          continue;
        }
        let trailers_tuple = (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY);
        let mut trailers_frame = HeadersFrame::new(trailers_tuple, stream_id);
        if data.is_empty() {
          trailers_frame.set_eos();
        }
        if trailers_rest.is_empty() {
          trailers_frame.set_eoh();
        }
        let trailers_init = init!(first_data, trailers_frame);
        write_array(
          [&headers_init, first_headers, &data_init, first_data, &trailers_init, first_trailers],
          *hdpm.is_conn_open,
          hdpm.stream,
        )
        .await?;
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        *data = data_rest;
        *trailers = trailers_rest;
        break;
      });
    }
    (true, false) => {
      return Err(crate::Error::http2_go_away_generic(Http2Error::TrailersWithoutData));
    }
    (true, true) => {
      process_receipt_loop!(hd, |guard| {
        let hdpm = guard.parts_mut();
        let (scrp, _wp);
        loop_checks!((scrp, _wp), hdpm, stream_id);
        let mut hf = HeadersFrame::new((hsreqh, hsresh), stream_id);
        if headers_rest.is_empty() {
          hf.set_eoh();
          hf.set_eos();
        }
        let headers_init = init!(first_headers, hf);
        write_array([&headers_init, first_headers], *hdpm.is_conn_open, hdpm.stream).await?;
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        break;
      });
    }
  }
  *headers = headers_rest;
  Ok(())
}

#[inline]
async fn slow_path<HB, HD, S, SB, const IS_CLIENT: bool>(
  (headers, data, trailers): (&mut &[u8], &mut &[u8], &mut &[u8]),
  hd: &HD,
  max_frame_len: usize,
  stream_id: U31,
  mut cb: impl FnMut(Http2DataPartsMut<'_, S, SB>),
) -> crate::Result<()>
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
  process_receipt_loop!(hd, |guard| {
    let hdpm = guard.parts_mut();
    let (scrp, mut wp);
    loop_checks!((scrp, wp), hdpm, stream_id);

    while let (before @ [_is_not_empty, ..], after) = split_frame_bytes(headers, max_frame_len) {
      let mut frame = ContinuationFrame::new(stream_id);
      if after.is_empty() {
        frame.set_eoh();
      }
      let init = init!(before, frame);
      write_array([&init, before], *hdpm.is_conn_open, hdpm.stream).await?;
      *headers = after;
    }

    while let (before @ [_is_not_empty, ..], after) = split_frame_bytes(data, max_frame_len) {
      let len = data_frame_len(before);
      if !wp.manage_send(len) {
        continue;
      }
      let mut frame = DataFrame::new(len, stream_id);
      if trailers.is_empty() && after.is_empty() {
        frame.set_eos();
      }
      let init = init!(before, frame);
      write_array([&init, before], *hdpm.is_conn_open, hdpm.stream).await?;
      *data = after;
    }

    if let (before @ [_is_not_empty, ..], after) = split_frame_bytes(trailers, max_frame_len) {
      let trailers_tuple = (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY);
      let mut frame = HeadersFrame::new(trailers_tuple, stream_id);
      frame.set_eos();
      if after.is_empty() {
        frame.set_eoh();
      }
      let init = init!(before, frame);
      write_array([&init, before], *hdpm.is_conn_open, hdpm.stream).await?;
      *trailers = after;
    }
    while let (before @ [_is_not_empty, ..], after) = split_frame_bytes(trailers, max_frame_len) {
      let mut frame = ContinuationFrame::new(stream_id);
      if after.is_empty() {
        frame.set_eoh();
      }
      let init = init!(before, frame);
      write_array([&init, before], *hdpm.is_conn_open, hdpm.stream).await?;
      *trailers = after;
    }

    change_final_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
    cb(hdpm);

    return Ok(());
  });
}

#[inline]
fn split_frame_bytes(bytes: &[u8], max_frame_len: usize) -> (&[u8], &[u8]) {
  if bytes.len() <= max_frame_len {
    (bytes, &[])
  } else {
    (bytes.get(..max_frame_len).unwrap_or_default(), bytes.get(max_frame_len..).unwrap_or_default())
  }
}
