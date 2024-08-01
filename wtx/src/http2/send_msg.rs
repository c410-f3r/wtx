//! A message is composed by header frames, data frames and trailer frames.
//!
//! 1. Header (1), Continuation (0+)
//! 2. Data (0+)
//! 3. Trailer (0 | 1), Continuation (0+)
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
  http::{Headers, ReqResBuffer},
  http2::{
    http2_data::Http2DataPartsMut,
    misc::{protocol_err, write_array},
    window::WindowsPair,
    ContinuationFrame, DataFrame, HeadersFrame, HpackEncoder, HpackStaticRequestHeaders,
    HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2Error, StreamState, U31,
  },
  misc::{LeaseMut, Lock, RefCounter, Stream, Usize, Vector},
};

#[inline]
pub(crate) async fn send_msg<HB, HD, RRB, S, const IS_CLIENT: bool>(
  data_bytes: &[u8],
  hd: &HD,
  headers: &Headers,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  mut cb: impl FnMut(Http2DataPartsMut<'_, RRB, S>),
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  let (mut has_headers, mut has_data) = (false, false);
  process_higher_operation!(
    hd,
    |guard| do_send_msg::<_, _, IS_CLIENT>(
      data_bytes,
      (&mut has_headers, &mut has_data),
      headers,
      guard.parts_mut(),
      (hsreqh, hsresh),
      stream_id,
      &mut cb,
    )
    .await,
    |_guard, elem| Ok(elem)
  )
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

// Tries to at least send initial headers when the windows size does not allow sending data frames
#[inline]
async fn do_send_msg<RRB, S, const IS_CLIENT: bool>(
  mut data_bytes: &[u8],
  (has_headers, has_data): (&mut bool, &mut bool),
  headers: &Headers,
  mut hdpm: Http2DataPartsMut<'_, RRB, S>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  cb: &mut impl FnMut(Http2DataPartsMut<'_, RRB, S>),
) -> crate::Result<Option<()>>
where
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  let Http2Buffer { hpack_enc, hpack_enc_buffer, scrp, .. } = &mut hdpm.hb;
  let Some(scrp) = scrp.get_mut(&stream_id) else {
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
        if write_standalone_headers::<S, IS_CLIENT>(
          data_bytes,
          headers,
          (hpack_enc, hpack_enc_buffer),
          (hsreqh, hsresh),
          *hdpm.is_conn_open,
          hdpm.hps.max_frame_len,
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
      if fast_path::<S, IS_CLIENT>(
        available_send,
        data_bytes,
        headers,
        (hpack_enc, hpack_enc_buffer),
        (hsreqh, hsresh),
        *hdpm.is_conn_open,
        hdpm.hps.max_frame_len,
        hdpm.stream,
        stream_id,
        &mut wp,
      )
      .await?
      {
        break 'msg;
      }
      change_initial_stream_state::<IS_CLIENT>(&mut scrp.stream_state);
      *has_headers = true;
    }

    if !*has_data {
      if write_standalone_data(
        available_send,
        has_data,
        &mut data_bytes,
        headers.has_trailers(),
        *hdpm.is_conn_open,
        hdpm.hps.max_frame_len,
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

    write_standalone_trailers(
      headers,
      (hpack_enc, hpack_enc_buffer),
      (hsreqh, hsresh),
      *hdpm.is_conn_open,
      hdpm.hps.max_frame_len,
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
fn encode_headers<const IS_CLIENT: bool>(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
) -> crate::Result<()> {
  hpack_enc_buffer.clear();
  if headers.has_trailers() {
    let iter = headers.iter().filter(|el| !el.is_trailer);
    if IS_CLIENT {
      hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), iter)?;
    } else {
      hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), iter)?;
    }
  } else if IS_CLIENT {
    hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), headers.iter())?;
  } else {
    hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), headers.iter())?;
  }
  Ok(())
}

#[inline]
fn encode_trailers(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
) -> crate::Result<()> {
  hpack_enc_buffer.clear();
  hpack_enc.encode(hpack_enc_buffer, [].into_iter(), headers.iter().filter(|el| el.is_trailer))?;
  Ok(())
}

// Tries to send everything in a single round trip. If not possible, will at least send headers.
#[inline]
async fn fast_path<S, const IS_CLIENT: bool>(
  available_send: u32,
  data_bytes: &[u8],
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
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
  fn has_valid(slice: &[u8], len: u32) -> Option<U31> {
    if !slice.is_empty() && slice.len() <= *Usize::from(len) {
      return Some(U31::from_u32(u32::try_from(slice.len()).ok()?));
    }
    None
  }

  encode_headers::<IS_CLIENT>(headers, (hpack_enc, hpack_enc_buffer), (hsreqh, hsresh))?;

  'headers_with_others: {
    let Some(data_len) = has_valid(data_bytes, available_send) else {
      break 'headers_with_others;
    };
    let Some(_) = has_valid(&hpack_enc_buffer, available_send) else {
      break 'headers_with_others;
    };
    if headers.has_trailers() {
      let idx = hpack_enc_buffer.len();
      encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
      let Some((headers_bytes, trailers_bytes)) = hpack_enc_buffer.split_at_checked(idx) else {
        break 'headers_with_others;
      };
      let Some(_) = has_valid(trailers_bytes, available_send) else {
        break 'headers_with_others;
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
    } else {
      let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
      let mut frame1 = DataFrame::new(data_len, stream_id);
      frame0.set_eoh();
      frame1.set_eos();
      write_array(
        [
          &init!(hpack_enc_buffer, frame0),
          hpack_enc_buffer,
          &init!(data_bytes, frame1),
          data_bytes,
        ],
        is_conn_open,
        stream,
      )
      .await?;
    }
    wp.withdrawn_send(Some(stream_id), data_len)?;
    return Ok(true);
  }

  write_standalone_headers::<_, IS_CLIENT>(
    data_bytes,
    headers,
    (hpack_enc, hpack_enc_buffer),
    (hsreqh, hsresh),
    is_conn_open,
    max_frame_len,
    stream,
    stream_id,
  )
  .await
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

/// Tries to send up two data frames in a single round trip. If exhausted, returns `true`.
#[inline]
async fn write_standalone_data<S>(
  available_send: u32,
  has_data: &mut bool,
  data_bytes: &mut &[u8],
  has_trailers: bool,
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
    has_trailers: bool,
  ) -> bool {
    if data.is_empty() {
      *has_data = true;
      if !has_trailers {
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
      let should_stop = should_stop(right1, &mut frame1, has_data, has_trailers);
      write_array(
        [&init!(left0, frame0), left0, &init!(left1, frame1), left1],
        is_conn_open,
        stream,
      )
      .await?;
      wp.withdrawn_send(Some(stream_id), frame0_len.wrapping_add(frame1_len))?;
      *data_bytes = right1;
      Ok(should_stop)
    } else {
      let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
      write_array([&init!(left0, frame0), left0], is_conn_open, stream).await?;
      wp.withdrawn_send(Some(stream_id), frame0_len)?;
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
    let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
    write_array([&init!(left0, frame0), left0], is_conn_open, stream).await?;
    wp.withdrawn_send(Some(stream_id), frame0_len)?;
    *data_bytes = right0;
    Ok(should_stop)
  }
}

// Tries to send all initial headers
#[inline]
async fn write_standalone_headers<S, const IS_CLIENT: bool>(
  data_bytes: &[u8],
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<bool>
where
  S: Stream,
{
  encode_headers::<IS_CLIENT>(headers, (hpack_enc, hpack_enc_buffer), (hsreqh, hsresh))?;
  let (left0, right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len);
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

/// Tries to send all trailer headers
#[inline]
async fn write_standalone_trailers<S>(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  max_frame_len: u32,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<()>
where
  S: Stream,
{
  hpack_enc_buffer.clear();
  encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
  let (left0 @ [_, ..], right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len) else {
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
