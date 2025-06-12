//! A message is composed by header frames, data frames and trailer frames.
//!
//! 1. Header (1), Continuation (0+)
//! 2. Data (0+)
//! 3. Trailer (0 | 1), Continuation (0+)
//!
//! Control frames like Settings or `WindowUpdate` are out of scope.

macro_rules! init {
  ($frame_len:expr, $frame:expr) => {{
    let mut buffer = [0; 9];
    let [a, b, c, d, e, f, g, h, i] = &mut buffer;
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
    buffer
  }};
}

use crate::{
  collection::Vector,
  http::{Headers, Trailers},
  http2::{
    Http2Buffer, Http2Data, Http2Error, Http2SendStatus, SendDataMode,
    continuation_frame::ContinuationFrame,
    data_frame::DataFrame,
    headers_frame::HeadersFrame,
    hpack_encoder::HpackEncoder,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    http2_data::Http2DataPartsMut,
    misc::{process_higher_operation_err, protocol_err, scrp_mut, write_array},
    send_data_mode::SendDataModeBytes,
    stream_state::StreamState,
    u31::U31,
    window::WindowsPair,
  },
  misc::{LeaseMut, Usize},
  stream::StreamWriter,
  sync::{AtomicBool, Lock, RefCounter},
};
use core::{
  future::poll_fn,
  pin::pin,
  sync::atomic::Ordering,
  task::{Poll, Waker},
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

pub(crate) async fn send_msg<HB, HD, SW, const IS_CLIENT: bool>(
  mut data_bytes: &[u8],
  hd: &HD,
  headers: &Headers,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: &AtomicBool,
  stream_id: U31,
  mut cb: impl FnMut(Http2DataPartsMut<'_, SW, IS_CLIENT>),
) -> crate::Result<Http2SendStatus>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SW: StreamWriter,
{
  let (mut has_headers, mut has_data) = (false, false);

  let mut lock_pin = pin!(hd.lock());
  let rslt = poll_fn(move |cx| {
    if !is_conn_open.load(Ordering::Relaxed) {
      return Poll::Ready(Ok(Http2SendStatus::ClosedConnection));
    }
    let mut lock = lock_pin!(cx, hd, lock_pin);
    let hdpm = lock.parts_mut();
    let fut = do_send_msg::<_, IS_CLIENT>(
      &mut data_bytes,
      (&mut has_headers, &mut has_data),
      headers,
      hdpm,
      (hsreqh, hsresh),
      stream_id,
      cx.waker(),
      &mut cb,
    );
    if let Poll::Ready(rslt) = pin!(fut).poll(cx) {
      if let Some(is_fully_sent) = rslt? {
        if is_fully_sent {
          if IS_CLIENT {
            _trace!("Request has been sent");
          } else {
            _trace!("Response has been sent");
          };
          return Poll::Ready(Ok(Http2SendStatus::Ok));
        }
      } else {
        return Poll::Ready(Ok(Http2SendStatus::ClosedStream));
      }
    }
    Poll::Pending
  })
  .await;
  if let Err(err) = &rslt {
    process_higher_operation_err(err, hd).await;
  }
  rslt
}

/// Tries to send up two data frames in a single round trip. If exhausted, returns `true`.
pub(crate) async fn write_standalone_data<'bytes, B, SW, const IS_SCATTERED: bool>(
  available_send: u32,
  data: &mut SendDataMode<B, IS_SCATTERED>,
  force_eos: bool,
  has_data: &mut bool,
  has_trailers: bool,
  is_conn_open: &AtomicBool,
  max_frame_len: u32,
  stream: &mut SW,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<bool>
where
  B: SendDataModeBytes<'bytes, IS_SCATTERED>,
  SW: StreamWriter,
{
  fn should_stop(
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
  if IS_SCATTERED {
    if let Some(available_send_rest @ 1..=u32::MAX) = available_send.checked_sub(max_frame_len) {
      let tuple = split_frame_bytes(data.first_mut(), max_frame_len);
      let (left0 @ [_, ..], right0) = tuple else {
        *has_data = true;
        return Ok(false);
      };
      let frame0_len = data_frame_len(left0.len());
      let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
      let split_len = max_frame_len.min(available_send_rest);
      if let (left1 @ [_, ..], right1) = split_frame_bytes(right0, split_len) {
        let frame1_len = data_frame_len(left1.len());
        let mut frame1 = DataFrame::new(frame1_len.into(), stream_id);
        let should_stop = should_stop(right1, &mut frame1, has_data, has_trailers);
        if force_eos {
          frame1.set_eos();
        }
        write_array(
          [&init!(frame0_len, frame0), left0, &init!(frame1_len, frame1), left1],
          is_conn_open,
          stream,
        )
        .await?;
        wp.withdrawn_send(Some(stream_id), frame0_len.wrapping_add(frame1_len).into())?;
        *data.first_mut() = right1;
        Ok(should_stop)
      } else {
        let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
        if force_eos {
          frame0.set_eos();
        }
        write_array([&init!(frame0_len, frame0), left0], is_conn_open, stream).await?;
        wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
        *data.first_mut() = right0;
        Ok(should_stop)
      }
    } else {
      let tuple = split_frame_bytes(data.first_mut(), available_send);
      let (left0 @ [_, ..], right0) = tuple else {
        *has_data = true;
        return Ok(false);
      };
      let frame0_len = data_frame_len(left0.len());
      let mut frame0 = DataFrame::new(frame0_len.into(), stream_id);
      let should_stop = should_stop(right0, &mut frame0, has_data, has_trailers);
      if force_eos {
        frame0.set_eos();
      }
      write_array([&init!(frame0_len, frame0), left0], is_conn_open, stream).await?;
      wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
      *data.first_mut() = right0;
      Ok(should_stop)
    }
  } else {
    let data_len = data.len();
    if data_len >= *Usize::from(max_frame_len) || data_len >= *Usize::from(available_send) {
      return Err(protocol_err(Http2Error::InvalidDataFrameDataLen));
    }
    let frame0_len = data_frame_len(data_len);
    let frame0 = DataFrame::new(frame0_len.into(), stream_id);
    write_array(data.concat(&init!(frame0_len, frame0)), is_conn_open, stream).await?;
    wp.withdrawn_send(Some(stream_id), frame0_len.into())?;
    *has_data = true;
    Ok(false)
  }
}

// Tries to send all initial headers
pub(crate) async fn write_standalone_headers<SW, const IS_CLIENT: bool>(
  hpack_enc_buffer: &mut Vector<u8>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: &AtomicBool,
  is_eos: bool,
  max_frame_len: u32,
  stream: &mut SW,
  stream_id: U31,
) -> crate::Result<bool>
where
  SW: StreamWriter,
{
  let (left0, right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len);
  let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
  let should_stop = if is_eos {
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
pub(crate) async fn write_standalone_trailers<SW>(
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  is_conn_open: &AtomicBool,
  max_frame_len: u32,
  stream: &mut SW,
  stream_id: U31,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  hpack_enc_buffer.clear();
  encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
  let (left0 @ [_, ..], right0) = split_frame_bytes(hpack_enc_buffer, max_frame_len) else {
    return Ok(());
  };
  let mut frame0 = HeadersFrame::new(
    (HpackStaticRequestHeaders::EMPTY, HpackStaticResponseHeaders::EMPTY),
    stream_id,
  );
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

// Tries to at least send initial headers when the windows size does not allow sending data frames
async fn do_send_msg<SW, const IS_CLIENT: bool>(
  data_bytes: &mut &[u8],
  (has_headers, has_data): (&mut bool, &mut bool),
  headers: &Headers,
  hdpm: Http2DataPartsMut<'_, SW, IS_CLIENT>,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  stream_id: U31,
  waker: &Waker,
  cb: &mut impl FnMut(Http2DataPartsMut<'_, SW, IS_CLIENT>),
) -> crate::Result<Option<bool>>
where
  SW: StreamWriter,
{
  let Http2Buffer { hpack_enc, hpack_enc_buffer, is_conn_open, scrp, .. } = hdpm.hb;
  let elem = scrp_mut(scrp, stream_id)?;
  if !elem.is_stream_open {
    return Ok(None);
  }
  if !elem.stream_state.can_send::<IS_CLIENT>() {
    return Err(protocol_err(Http2Error::InvalidSendStreamState));
  }
  let mut wp = WindowsPair::new(hdpm.windows, &mut elem.windows);

  'msg: {
    let Ok(available_send @ 1..=u32::MAX) = u32::try_from(wp.available_send()) else {
      if !*has_headers {
        encode_headers::<IS_CLIENT>(headers, (hpack_enc, hpack_enc_buffer), (hsreqh, hsresh))?;
        if write_standalone_headers::<SW, IS_CLIENT>(
          hpack_enc_buffer,
          (hsreqh, hsresh),
          is_conn_open,
          data_bytes.is_empty(),
          hdpm.hps.max_frame_len,
          hdpm.stream_writer,
          stream_id,
        )
        .await?
        {
          break 'msg;
        }
        change_initial_stream_state::<IS_CLIENT>(&mut elem.stream_state);
        *has_headers = true;
      }
      return Ok(Some(false));
    };

    if !*has_headers {
      if fast_path::<SW, IS_CLIENT>(
        available_send,
        data_bytes,
        headers,
        (hpack_enc, hpack_enc_buffer),
        (hsreqh, hsresh),
        is_conn_open,
        hdpm.hps.max_frame_len,
        hdpm.stream_writer,
        stream_id,
        &mut wp,
      )
      .await?
      {
        break 'msg;
      }
      change_initial_stream_state::<IS_CLIENT>(&mut elem.stream_state);
      *has_headers = true;
    }

    if !*has_data {
      if write_standalone_data(
        available_send,
        &mut SendDataMode::scattered_data_frames(data_bytes),
        false,
        has_data,
        headers.trailers().has_any(),
        is_conn_open,
        hdpm.hps.max_frame_len,
        hdpm.stream_writer,
        stream_id,
        &mut wp,
      )
      .await?
      {
        break 'msg;
      }
      // There can be an available window size
      waker.wake_by_ref();
      return Ok(Some(false));
    }

    write_standalone_trailers(
      headers,
      (hpack_enc, hpack_enc_buffer),
      is_conn_open,
      hdpm.hps.max_frame_len,
      hdpm.stream_writer,
      stream_id,
    )
    .await?;
  }
  change_final_stream_state::<IS_CLIENT>(&mut elem.stream_state);
  cb(hdpm);
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
async fn fast_path<SW, const IS_CLIENT: bool>(
  available_send: u32,
  data_bytes: &[u8],
  headers: &Headers,
  (hpack_enc, hpack_enc_buffer): (&mut HpackEncoder, &mut Vector<u8>),
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: &AtomicBool,
  max_frame_len: u32,
  stream: &mut SW,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<bool>
where
  SW: StreamWriter,
{
  fn has_delimited_bytes(data_bytes: &[u8], len: u32) -> Option<U31> {
    if !data_bytes.is_empty() && data_bytes.len() <= *Usize::from(len) {
      return Some(U31::from_u32(u32::try_from(data_bytes.len()).ok()?));
    }
    None
  }

  encode_headers::<IS_CLIENT>(headers, (hpack_enc, hpack_enc_buffer), (hsreqh, hsresh))?;

  'headers_with_others: {
    let Some(data_len) = has_delimited_bytes(data_bytes, available_send.min(max_frame_len)) else {
      break 'headers_with_others;
    };

    if headers.trailers().has_any() {
      let idx = hpack_enc_buffer.len();
      encode_trailers(headers, (hpack_enc, hpack_enc_buffer))?;
      let Some((headers_bytes, trailers_bytes)) = hpack_enc_buffer.split_at_checked(idx) else {
        break 'headers_with_others;
      };
      let Some(_) = has_delimited_bytes(headers_bytes, max_frame_len) else {
        break 'headers_with_others;
      };
      let Some(_) = has_delimited_bytes(trailers_bytes, max_frame_len) else {
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
          &init!(data_frame_len(headers_bytes.len()), frame0),
          headers_bytes,
          &init!(data_frame_len(data_bytes.len()), frame1),
          data_bytes,
          &init!(data_frame_len(trailers_bytes.len()), frame2),
          trailers_bytes,
        ],
        is_conn_open,
        stream,
      )
      .await?;
    } else {
      let Some(_) = has_delimited_bytes(hpack_enc_buffer, max_frame_len) else {
        break 'headers_with_others;
      };
      let mut frame0 = HeadersFrame::new((hsreqh, hsresh), stream_id);
      let mut frame1 = DataFrame::new(data_len, stream_id);
      frame0.set_eoh();
      frame1.set_eos();
      write_array(
        [
          &init!(data_frame_len(hpack_enc_buffer.len()), frame0),
          hpack_enc_buffer,
          &init!(data_frame_len(data_bytes.len()), frame1),
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
    hpack_enc_buffer,
    (hsreqh, hsresh),
    is_conn_open,
    data_bytes.is_empty(),
    max_frame_len,
    stream,
    stream_id,
  )
  .await
}

fn split_frame_bytes(bytes: &[u8], len: u32) -> (&[u8], &[u8]) {
  let n = *Usize::from(len);
  (bytes.get(..n).unwrap_or(bytes), bytes.get(n..).unwrap_or_default())
}

async fn write_headers_or_trailers<SW>(
  frame0: &mut HeadersFrame<'_>,
  is_conn_open: &AtomicBool,
  (left0, right0): (&[u8], &[u8]),
  max_frame_len: u32,
  stream: &mut SW,
  stream_id: U31,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  if let (left1 @ [_, ..], right1) = split_frame_bytes(right0, max_frame_len) {
    let mut frame1 = ContinuationFrame::new(stream_id);
    if !right1.is_empty() {
      return Err(protocol_err(Http2Error::HeadersOverflow));
    }
    frame1.set_eoh();
    write_array(
      [
        &init!(data_frame_len(left0.len()), frame0),
        left0,
        &init!(data_frame_len(left1.len()), frame1),
        left1,
      ],
      is_conn_open,
      stream,
    )
    .await?;
  } else {
    frame0.set_eoh();
    write_array([&init!(data_frame_len(left0.len()), frame0), left0], is_conn_open, stream).await?;
  }
  Ok(())
}
