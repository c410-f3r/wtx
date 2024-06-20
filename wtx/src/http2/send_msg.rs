//! A message is composed by header frames, data frames and trailer frames.
//!
//! 1. Header (1+), Continuation (0+)
//! 2. Data (0+)
//! 3. Trailer (0+), Continuation (0+)
//!
//! Control frames like Settings or WindowUpdate are out of scope.
macro_rules! init {
  ($bytes:expr, $frame:expr) => {{
    let mut buffer = [0; 9];
    let [a, b, c, d, e, f, g, h, i] = &mut buffer;
    let [_, j, k, l] = u32::try_from($bytes.len()).unwrap_or_default().to_be_bytes();
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
    buffer
  }};
}

use crate::{
  http::{Header, Headers},
  http2::{
    http2_data::Http2DataPartsMut,
    misc::{maybe_send_based_on_error, protocol_err, write_array},
    window::WindowsPair,
    ContinuationFrame, DataFrame, HeadersFrame, HpackEncoder, HpackHeaderBasic,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2Error,
    StreamBuffer, StreamState, U31,
  },
  misc::{ByteVector, LeaseMut, Lock, RefCounter, Stream, Usize},
};
use tokio::sync::MutexGuard;

#[inline]
pub(crate) async fn send_msg<HB, HD, S, SB, const IS_CLIENT: bool>(
  data_bytes: &[u8],
  hd: &HD,
  headers: &Headers,
  hpack_enc_buffer: &mut ByteVector,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
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
  let (max_frame_len, headers_bytes, trailers_bytes) = {
    let mut guard = hd.lock().await;
    let hdpm = guard.parts_mut();
    let rslt = if IS_CLIENT {
      encode_headers(headers, &mut hdpm.hb.hpack_enc, hpack_enc_buffer, hsreqh.iter())
    } else {
      encode_headers(headers, &mut hdpm.hb.hpack_enc, hpack_enc_buffer, hsresh.iter())
    };
    let max_frame_len = hdpm.hps.max_frame_len;
    let (headers_bytes, trailers_bytes) = maybe_send_based_on_error(rslt, hdpm).await?;
    (max_frame_len, headers_bytes, trailers_bytes)
  };
  let (mut has_headers, mut has_data) = (false, false);
  process_higher_operation!(hd, |guard| {
    do_send_msg::<_, _, IS_CLIENT>(
      (headers_bytes, data_bytes, trailers_bytes),
      (&mut has_headers, &mut has_data),
      guard.parts_mut(),
      (hsreqh, hsresh),
      max_frame_len,
      stream_id,
      &mut cb,
    )
    .await
  });
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
async fn do_send_msg<S, SB, const IS_CLIENT: bool>(
  (headers_bytes, mut data_bytes, trailers_bytes): (&[u8], &[u8], &[u8]),
  (has_headers, has_data): (&mut bool, &mut bool),
  hdpm: Http2DataPartsMut<'_, S, SB>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  max_frame_len: u32,
  stream_id: U31,
  cb: &mut impl FnMut(Http2DataPartsMut<'_, S, SB>),
) -> crate::Result<Option<()>>
where
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  let Some(scrp) = hdpm.hb.scrp.get_mut(&stream_id) else {
    return Err(protocol_err(Http2Error::BadLocalFlow));
  };
  if !scrp.stream_state.can_send_stream::<IS_CLIENT>() {
    return Err(protocol_err(Http2Error::InvalidSendStreamState));
  }
  let mut wp = WindowsPair::new(hdpm.windows, &mut scrp.windows);

  'msg: {
    let available_send = if let Ok(elem @ 1..=u32::MAX) = u32::try_from(wp.available_send()) {
      elem
    } else {
      if !*has_headers {
        if write_headers(
          (headers_bytes, data_bytes),
          (hsreqh, hsresh),
          *hdpm.is_conn_open,
          max_frame_len,
          hdpm.stream,
          stream_id,
        )
        .await?
        {
          break 'msg;
        }
        change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
        *has_headers = true;
      }
      return Ok(None);
    };

    if !*has_headers {
      if fast_path(
        available_send,
        (headers_bytes, data_bytes, trailers_bytes),
        (hsreqh, hsresh),
        *hdpm.is_conn_open,
        max_frame_len,
        hdpm.stream,
        stream_id,
      )
      .await?
      {
        break 'msg;
      }
      if write_headers(
        (headers_bytes, data_bytes),
        (hsreqh, hsresh),
        *hdpm.is_conn_open,
        max_frame_len,
        hdpm.stream,
        stream_id,
      )
      .await?
      {
        break 'msg;
      }
      change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
      *has_headers = true;
    }

    if !*has_data {
      if write_data(
        available_send,
        has_data,
        (&mut data_bytes, trailers_bytes),
        *hdpm.is_conn_open,
        max_frame_len,
        hdpm.stream,
        stream_id,
        &mut wp,
      )
      .await?
      {
        break 'msg;
      }
      return Ok(None);
    }

    write_trailers(
      trailers_bytes,
      (hsreqh, hsresh),
      *hdpm.is_conn_open,
      max_frame_len,
      hdpm.stream,
      stream_id,
    )
    .await?;
  }

  change_final_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
  cb(hdpm);
  Ok(Some(()))
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
async fn fast_path<S>(
  available_send: u32,
  (headers_bytes, data_bytes, trailers_bytes): (&[u8], &[u8], &[u8]),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<bool>
where
  S: Stream,
{
  #[inline]
  fn has_valid(slice: &[u8], len: u32) -> Option<U31> {
    if slice.len() > 0 && slice.len() <= *Usize::from(len) {
      return Some(U31::from_u32(u32::try_from(slice.len()).ok()?));
    }
    None
  }
  let has_valid_headers = has_valid(headers_bytes, max_frame_len);
  let has_valid_data = has_valid(data_bytes, available_send);
  let has_valid_trailers = has_valid(trailers_bytes, max_frame_len);
  match (has_valid_headers, has_valid_data, has_valid_trailers) {
    (Some(_), None, None) if data_bytes.is_empty() && trailers_bytes.is_empty() => {
      let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
      frame0.set_eoh();
      frame0.set_eos();
      write_array([&init!(headers_bytes, frame0), headers_bytes], is_conn_open, stream).await?;
      Ok(true)
    }
    (Some(_), Some(data_len), None) if trailers_bytes.is_empty() => {
      let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
      frame0.set_eoh();
      let mut frame1 = DataFrame::new(data_len, stream_id);
      frame1.set_eos();
      write_array(
        [&init!(headers_bytes, frame0), headers_bytes, &init!(data_bytes, frame1), data_bytes],
        is_conn_open,
        stream,
      )
      .await?;
      Ok(true)
    }
    (Some(_), Some(data_len), Some(_)) => {
      let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
      frame0.set_eoh();
      let frame1 = DataFrame::new(data_len, stream_id);
      let mut frame2 = HeadersFrame::new(
        (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY),
        stream_id,
      );
      frame2.set_eoh();
      frame2.set_eos();
      write_array(
        [
          &init!(headers_bytes, frame0),
          headers_bytes,
          &init!(data_bytes, frame1),
          data_bytes,
          &init!(trailers_bytes, frame2),
          trailers_bytes,
        ],
        is_conn_open,
        stream,
      )
      .await?;
      Ok(true)
    }
    _ => Ok(false),
  }
}

#[inline]
fn split_frame_bytes(bytes: &[u8], len: u32) -> (&[u8], &[u8]) {
  let n = *Usize::from(len);
  if n >= bytes.len() {
    (bytes, &[])
  } else {
    (bytes.get(..n).unwrap_or_default(), bytes.get(n..).unwrap_or_default())
  }
}

#[inline]
async fn write_data<S>(
  available_send: u32,
  has_data: &mut bool,
  (data_bytes, trailers_bytes): (&mut &[u8], &[u8]),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<bool>
where
  S: Stream,
{
  #[inline]
  fn should_stop(
    data: &[u8],
    frame: &mut DataFrame,
    has_data: &mut bool,
    trailers_bytes: &[u8],
  ) -> bool {
    if data.is_empty() {
      *has_data = true;
      if trailers_bytes.is_empty() {
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
    let (left0 @ [_, ..], right0) = split_frame_bytes(data_bytes, max_frame_len) else {
      *has_data = true;
      return Ok(false);
    };
    let frame0_len = data_frame_len(left0);
    let mut frame0 = DataFrame::new(frame0_len, stream_id);
    let split_len = max_frame_len.min(available_send_rest);
    if let (left1 @ [_, ..], right1) = split_frame_bytes(right0, split_len) {
      let mut frame1 = DataFrame::new(data_frame_len(left1), stream_id);
      let frame1_len = data_frame_len(left1);
      let should_stop = should_stop(right1, &mut frame1, has_data, trailers_bytes);
      write_array(
        [&init!(left0, frame0), left0, &init!(left1, frame1), left1],
        is_conn_open,
        stream,
      )
      .await?;
      wp.withdrawn_send(frame0_len.wrapping_add(frame1_len));
      *data_bytes = right1;
      Ok(should_stop)
    } else {
      let should_stop = should_stop(right0, &mut frame0, has_data, trailers_bytes);
      write_array([&init!(left0, frame0), left0], is_conn_open, stream).await?;
      wp.withdrawn_send(frame0_len);
      *data_bytes = right0;
      Ok(should_stop)
    }
  } else {
    let (left0 @ [_, ..], right0) = split_frame_bytes(data_bytes, available_send) else {
      *has_data = true;
      return Ok(false);
    };
    let frame0_len = data_frame_len(left0);
    let mut frame0 = DataFrame::new(frame0_len, stream_id);
    let should_stop = should_stop(right0, &mut frame0, has_data, trailers_bytes);
    write_array([&init!(left0, frame0), left0], is_conn_open, stream).await?;
    wp.withdrawn_send(frame0_len);
    *data_bytes = right0;
    Ok(should_stop)
  }
}

#[inline]
async fn write_headers<S>(
  (headers_bytes, data_bytes): (&[u8], &[u8]),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<bool>
where
  S: Stream,
{
  let (left0, right0) = split_frame_bytes(headers_bytes, max_frame_len);
  let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
  let should_stop = if data_bytes.is_empty() {
    frame0.set_eos();
    true
  } else {
    false
  };
  write_headers_or_trailers(
    &mut frame0,
    is_conn_open,
    (left0, right0),
    max_frame_len,
    stream,
    stream_id,
  )
  .await?;
  Ok(should_stop)
}

#[inline]
async fn write_headers_or_trailers<S>(
  frame0: &mut HeadersFrame<'_>,
  is_conn_open: bool,
  (left0, right0): (&[u8], &[u8]),
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<()>
where
  S: Stream,
{
  if let (left1 @ [_, ..], right1) = split_frame_bytes(right0, max_frame_len) {
    let mut frame1 = ContinuationFrame::new(stream_id);
    if !right1.is_empty() {
      return Err(protocol_err(Http2Error::HeadersOverflow));
    }
    frame1.set_eoh();
    let init1 = init!(left1, frame1);
    write_array([&init!(left0, frame0), left0, &init1, left1], is_conn_open, stream).await?;
  } else {
    frame0.set_eoh();
    write_array([&init!(left0, frame0), left0], is_conn_open, stream).await?;
  }
  Ok(())
}

#[inline]
async fn write_trailers<S>(
  trailers_bytes: &[u8],
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<()>
where
  S: Stream,
{
  let (left0 @ [_, ..], right0) = split_frame_bytes(trailers_bytes, max_frame_len) else {
    return Ok(());
  };
  let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
  frame0.set_eos();
  write_headers_or_trailers(
    &mut frame0,
    is_conn_open,
    (left0, right0),
    max_frame_len,
    stream,
    stream_id,
  )
  .await?;
  Ok(())
}
