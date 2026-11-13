// * NB = Network Buffer
// * CB = Compression Buffer
// * UB = User Buffer
//
// |    Frame   |   With Decompression   | Without Decompression |
// |------------|------------------------|-----------------------|
// |Single      |(NB -> UB)¹             |(NB)¹                  |
// |Continuation|(NB -> CB)¹⁺ (CB -> UB)¹|(NB -> UB)¹⁺           |

use crate::{
  codec::DecompressionFlush,
  collections::{ArrayVectorCopy, VectorU8, VectorUsize},
  futures::FnMutFut,
  misc::{ExtUtf8Error, PartialChar, from_utf8_basic, from_utf8_ext, process_utf8_stream},
  net::{BufStreamReader, StreamReader, StreamWriter},
  rng::Rng,
  web_socket::{
    CloseCode, Frame, FrameMut, MAX_CONTROL_PAYLOAD_LEN, OpCode, WebSocketError,
    WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    mask_op::mask_op,
    misc::{manage_read_close_frame, write_control_frame, write_control_frame_cb},
    read_frame_info::ReadFrameInfo,
    web_socket_bridge::WebSocketBridge,
    web_socket_compression::WebSocketDecompression,
  },
};
use core::{hint::cold_path, iter, mem, range::Range, slice};

const DECOMPRESSION_SUFFIX: [u8; 4] = [0, 0, 255, 255];

/// Returns `true` if a control frame was received.
pub(crate) async fn manage_auto_reply<A, RNG, const HAS_AUTO_REPLY: bool, const IS_CLIENT: bool>(
  aux: A,
  no_masking: bool,
  op_code: OpCode,
  payload: &mut [u8],
  rng: &mut RNG,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  write_control_frame_cb: impl for<'any> FnMutFut<
    (A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<bool>
where
  RNG: Rng,
{
  match op_code {
    OpCode::Close => {
      let mut control_payload = ArrayVectorCopy::<_, MAX_CONTROL_PAYLOAD_LEN>::try_from(&*payload)?;
      if manage_read_close_frame(CloseCode::Protocol, &mut control_payload)? {
        return Err(WebSocketError::InvalidCloseFrame.into());
      }
      if HAS_AUTO_REPLY {
        write_control_frame::<_, _, IS_CLIENT>(
          aux,
          no_masking,
          OpCode::Close,
          &mut control_payload,
          rng,
          write_control_frame_cb,
        )
        .await?;
      } else {
        stream_bridge.update((OpCode::Close, control_payload));
      }
      Ok(true)
    }
    OpCode::Ping => {
      let mut control_payload = ArrayVectorCopy::<_, MAX_CONTROL_PAYLOAD_LEN>::try_from(&*payload)?;
      if HAS_AUTO_REPLY {
        write_control_frame::<_, _, IS_CLIENT>(
          aux,
          no_masking,
          OpCode::Pong,
          &mut control_payload,
          rng,
          write_control_frame_cb,
        )
        .await?;
      } else {
        stream_bridge.update((OpCode::Pong, control_payload));
      }
      Ok(true)
    }
    OpCode::Pong => {
      let _control_payload = ArrayVectorCopy::<_, MAX_CONTROL_PAYLOAD_LEN>::try_from(&*payload)?;
      Ok(true)
    }
    OpCode::Binary | OpCode::Continuation | OpCode::Text => Ok(false),
  }
}

/// Returns `true` if `op_code` is a continuation frame and `fin` is also `true`.
pub(crate) fn manage_op_code_of_continuation_frames(
  fin: bool,
  first_op_code: OpCode,
  iuc: &mut Option<PartialChar>,
  op_code: OpCode,
  payload: &[u8],
  cb: &mut impl FnMut(&[u8], &mut Option<PartialChar>) -> crate::Result<()>,
) -> crate::Result<bool> {
  match op_code {
    OpCode::Continuation => {
      if first_op_code.is_text() {
        cb(payload, iuc)?;
      }
      if fin {
        if first_op_code.is_text() && iuc.is_some() {
          return Err(crate::Error::InvalidUTF8);
        }
        return Ok(true);
      }
    }
    OpCode::Binary | OpCode::Text => {
      return Err(WebSocketError::UnexpectedFrame.into());
    }
    OpCode::Close | OpCode::Ping | OpCode::Pong => {}
  }
  Ok(false)
}

pub(crate) fn manage_op_code_of_first_continuation_frame(
  op_code: OpCode,
  payload: &[u8],
  cb: fn(&[u8]) -> crate::Result<Option<PartialChar>>,
) -> crate::Result<Option<PartialChar>> {
  match op_code {
    OpCode::Binary => Ok(None),
    OpCode::Text => cb(payload),
    OpCode::Close | OpCode::Continuation | OpCode::Ping | OpCode::Pong => {
      Err(WebSocketError::UnexpectedFrame.into())
    }
  }
}

pub(crate) fn manage_op_code_of_first_final_frame(
  op_code: OpCode,
  payload: &[u8],
) -> crate::Result<()> {
  match op_code {
    OpCode::Continuation => {
      return Err(WebSocketError::UnexpectedFrame.into());
    }
    OpCode::Text => {
      let _str_validation = from_utf8_basic(payload)?;
    }
    OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong => {}
  }
  Ok(())
}

pub(crate) fn manage_text_of_first_continuation_frame(
  payload: &[u8],
) -> crate::Result<Option<PartialChar>> {
  Ok(match from_utf8_ext(payload) {
    Err(ExtUtf8Error::Incomplete(el)) => Some(el),
    Err(ExtUtf8Error::Invalid) => {
      return Err(crate::Error::InvalidUTF8);
    }
    Ok(_) => None,
  })
}

pub(crate) fn manage_text_of_recurrent_continuation_frames(
  curr_payload: &[u8],
  iuc: &mut Option<PartialChar>,
) -> crate::Result<()> {
  let _ = process_utf8_stream(iuc, curr_payload)?;
  Ok(())
}

async fn read_continuation_frames_until_final<
  'frame,
  'nb,
  'rb,
  'ub,
  D,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  force_user_buffer: bool,
  is_in_continuation_frame: &mut IsInContinuationFrame,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'nb mut BufStreamReader,
  no_masking: bool,
  payload_origin: WebSocketPayloadOrigin,
  reader_buffer: &'rb mut VectorUsize<u8>,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  user_buffer: &'ub mut VectorUsize<u8>,
  closed_conn_cb: &mut impl FnMut(&mut S),
  stream_reader_cb: &mut impl FnMut(&mut S) -> &mut SR,
  stream_writer_cb: &mut impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<(FrameMut<'frame>, bool)>
where
  'nb: 'frame,
  'rb: 'frame,
  'ub: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  user_buffer.truncate(is_in_continuation_frame.user_buffer_end);
  let is_decompressing = !D::IS_NOOP && is_in_continuation_frame.should_decompress;
  let control_frame = if is_decompressing {
    read_continuation_frames::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      &mut *reader_buffer,
      &mut *user_buffer,
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      rng,
      stream,
      stream_bridge,
      closed_conn_cb,
      &mut copy_from_compressed_rb1_to_rb2,
      &mut |_, _| Ok(()),
      stream_reader_cb,
      stream_writer_cb,
    )
    .await?
  } else {
    read_continuation_frames::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      user_buffer,
      reader_buffer,
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      rng,
      stream,
      stream_bridge,
      closed_conn_cb,
      &mut |_, _, _, _, _, _| Ok(()),
      &mut manage_text_of_recurrent_continuation_frames,
      stream_reader_cb,
      stream_writer_cb,
    )
    .await?
  };
  let is_final = control_frame.is_none();
  let (op_code, payload) = if let Some(op_code) = control_frame {
    let payload = match payload_origin {
      WebSocketPayloadOrigin::Adaptive => network_buffer.current_mut(),
      WebSocketPayloadOrigin::Consistent => {
        let user_buffer_begin = is_in_continuation_frame.user_buffer_begin;
        if is_decompressing {
          user_buffer.truncate(user_buffer_begin);
          user_buffer.extend_from_copyable_slice(network_buffer.current())?;
          user_buffer.as_slice_mut().get_mut(user_buffer_begin..).unwrap_or_default()
        } else if force_user_buffer {
          let control_begin = user_buffer.len();
          user_buffer.extend_from_copyable_slice(network_buffer.current())?;
          is_in_continuation_frame.user_buffer_end = control_begin;
          user_buffer.as_slice_mut().get_mut(control_begin..).unwrap_or_default()
        } else {
          reader_buffer.clear();
          reader_buffer.extend_from_copyable_slice(network_buffer.current())?;
          reader_buffer.as_slice_mut()
        }
      }
    };
    (op_code, payload)
  } else {
    reader_buffer.clear();
    let op_code = is_in_continuation_frame.op_code;
    let user_buffer_begin = is_in_continuation_frame.user_buffer_begin;
    (op_code, user_buffer.as_slice_mut().get_mut(user_buffer_begin..).unwrap_or_default())
  };
  Ok((Frame::new(true, op_code, payload, nc_rsv1), is_final))
}

/// In concurrent scenarios the close frame is forwarded through the bridge, so the state must be
/// updated before waking the writer. In sequential scenarios the automatic reply is written by
/// this task, so the state must only be updated after the write is performed. Otherwise writers
/// that ignore operations after closure silently drop the reply.
pub(crate) async fn read_frame<
  'frame,
  'nb,
  'rb,
  'ub,
  D,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  is_in_continuation_frame_opt: &mut Option<IsInContinuationFrame>,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'nb mut BufStreamReader,
  no_masking: bool,
  payload_origin: WebSocketPayloadOrigin,
  reader_buffer: &'rb mut VectorUsize<u8>,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  user_buffer: &'ub mut VectorUsize<u8>,
  mut closed_conn_cb: impl FnMut(&mut S),
  mut stream_reader_cb: impl FnMut(&mut S) -> &mut SR,
  mut stream_writer_cb: impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<FrameMut<'frame>>
where
  'nb: 'frame,
  'rb: 'frame,
  'ub: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  let Some(is_in_continuation_frame) = is_in_continuation_frame_opt else {
    user_buffer.clear();
    let first_rfi = fetch_frame_from_stream::<D, _, IS_CLIENT>(
      max_payload_len,
      nc_rsv1,
      network_buffer,
      no_masking,
      stream_reader_cb(&mut *stream),
    )
    .await?;
    return read_frame_from_first_rfi::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      false,
      is_in_continuation_frame_opt,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      payload_origin,
      reader_buffer,
      first_rfi,
      rng,
      stream,
      stream_bridge,
      user_buffer,
      &mut closed_conn_cb,
      &mut stream_reader_cb,
      &mut stream_writer_cb,
    )
    .await;
  };
  let tuple = read_continuation_frames_until_final::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
    false,
    is_in_continuation_frame,
    max_payload_len,
    nc,
    nc_rsv1,
    network_buffer,
    no_masking,
    payload_origin,
    reader_buffer,
    rng,
    stream,
    stream_bridge,
    user_buffer,
    &mut closed_conn_cb,
    &mut stream_reader_cb,
    &mut stream_writer_cb,
  )
  .await?;
  let (frame, is_final) = tuple;
  if is_final {
    *is_in_continuation_frame_opt = None;
  }
  Ok(frame)
}

pub(crate) async fn read_frames<
  'frame,
  'rb,
  CCB,
  D,
  R,
  S,
  SR,
  SRB,
  SW,
  SWB,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  is_in_continuation_frame_opt: &mut Option<IsInContinuationFrame>,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'frame mut BufStreamReader,
  no_masking: bool,
  reader_batches: &'frame mut VectorU8<(ReadFrameInfo, Range<usize>)>,
  reader_buffer: &'rb mut VectorUsize<u8>,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  user_buffer: &'frame mut VectorUsize<u8>,
  mut closed_conn_cb: CCB,
  mut stream_reader_cb: SRB,
  mut stream_writer_cb: SWB,
) -> crate::Result<
  impl Iterator<Item = FrameMut<'frame>>
  + 'frame
  + use<'frame, D, R, S, SR, SW, CCB, SRB, SWB, HAS_AUTO_REPLY, IS_CLIENT>,
>
where
  'rb: 'frame,
  CCB: FnMut(&mut S),
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SRB: FnMut(&mut S) -> &mut SR,
  SW: StreamWriter,
  SWB: FnMut(&mut S) -> &mut SW,
{
  if is_in_continuation_frame_opt.is_some() {
    let frame = read_frame::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      is_in_continuation_frame_opt,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      WebSocketPayloadOrigin::Consistent,
      reader_buffer,
      rng,
      stream,
      stream_bridge,
      user_buffer,
      &mut closed_conn_cb,
      &mut stream_reader_cb,
      &mut stream_writer_cb,
    )
    .await?;
    return Ok(ReadFramesIter::One(iter::once(frame)));
  }
  reader_batches.clear();
  user_buffer.clear();
  fetch_frames_from_stream::<D, _, IS_CLIENT>(
    max_payload_len,
    nc_rsv1,
    network_buffer,
    no_masking,
    reader_batches,
    stream_reader_cb(stream),
  )
  .await?;
  let mut processed = 0;
  let len = usize::from(reader_batches.len());
  for idx in 0..len {
    let Some((rfi, range)) = reader_batches.get(idx).copied() else {
      break;
    };
    network_buffer.set_indices(range.start, range.end);
    let mut payload_begin = user_buffer.len();
    let frame = read_frame_from_first_rfi::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      true,
      is_in_continuation_frame_opt,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      WebSocketPayloadOrigin::Consistent,
      reader_buffer,
      rfi,
      rng,
      stream,
      stream_bridge,
      user_buffer,
      &mut closed_conn_cb,
      &mut stream_reader_cb,
      &mut stream_writer_cb,
    )
    .await?;
    let frame_fin = frame.fin();
    let frame_op_code = frame.op_code();
    payload_begin = if !rfi.fin && frame_op_code.is_control() {
      is_in_continuation_frame_opt.as_ref().map_or(payload_begin, |el| el.user_buffer_end)
    } else {
      payload_begin
    };
    let payload_end = user_buffer.len();
    if let Some(entry) = reader_batches.as_slice_mut().get_mut(idx) {
      entry.0.fin = frame_fin;
      entry.0.op_code = frame_op_code;
      entry.1 = (payload_begin..payload_end).into();
    }
    processed = idx.wrapping_add(1);
    if !rfi.fin {
      break;
    }
  }
  Ok(ReadFramesIter::Many {
    batches: reader_batches.as_slice().get(..processed).unwrap_or_default().iter(),
    offset: 0,
    remaining: user_buffer.as_slice_mut(),
    rsv1: nc_rsv1,
  })
}

pub(crate) fn unmask_nb<const IS_CLIENT: bool>(
  mask: Option<[u8; 4]>,
  network_buffer: &mut [u8],
  no_masking: bool,
) -> crate::Result<()> {
  if !IS_CLIENT && !no_masking {
    mask_op(network_buffer, mask.ok_or(WebSocketError::MissingFrameMask)?);
  }
  Ok(())
}

fn copy_from_arbitrary_nb_to_rb1<const IS_CLIENT: bool>(
  mask: Option<[u8; 4]>,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  reader_buffer_first: &mut VectorUsize<u8>,
) -> crate::Result<()> {
  let current_mut = network_buffer.current_mut();
  unmask_nb::<IS_CLIENT>(mask, current_mut, no_masking)?;
  reader_buffer_first.extend_from_copyable_slice(current_mut)?;
  Ok(())
}

/// Only used when a single finished frame is received
fn copy_from_compressed_nb_to_rb1<D, const IS_CLIENT: bool>(
  nc: &mut D,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  reader_buffer: &mut VectorUsize<u8>,
  rfi: &ReadFrameInfo,
  upper_bound: usize,
) -> crate::Result<()>
where
  D: WebSocketDecompression,
{
  unmask_nb::<IS_CLIENT>(rfi.mask, network_buffer.current_mut(), no_masking)?;
  let _ = nc.decompress(
    DecompressionFlush::NoFlush,
    network_buffer.current(),
    reader_buffer,
    upper_bound,
  )?;
  let _ = nc.decompress(
    DecompressionFlush::SyncFlush,
    &DECOMPRESSION_SUFFIX,
    reader_buffer,
    upper_bound,
  )?;
  if !D::IS_NOOP && nc.no_context_takeover() {
    nc.reset();
  }
  Ok(())
}

/// All previously compressed continuation frames were concatenated into rb2. This is the
/// final decompression.
fn copy_from_compressed_rb1_to_rb2<D>(
  first_op_code: OpCode,
  max_payload_len: usize,
  nc: &mut D,
  user_buffer_begin: usize,
  reader_buffer_first: &mut VectorUsize<u8>,
  reader_buffer_second: &mut VectorUsize<u8>,
) -> crate::Result<()>
where
  D: WebSocketDecompression,
{
  reader_buffer_second.truncate(user_buffer_begin);
  reader_buffer_first.extend_from_copyable_slice(&DECOMPRESSION_SUFFIX)?;
  let payload_len = nc.decompress(
    DecompressionFlush::SyncFlush,
    reader_buffer_first,
    reader_buffer_second,
    max_payload_len.saturating_add(user_buffer_begin),
  )?;
  reader_buffer_second.truncate(user_buffer_begin.saturating_add(payload_len));
  if !D::IS_NOOP && nc.no_context_takeover() {
    nc.reset();
  }
  let message = reader_buffer_second.as_slice().get(user_buffer_begin..).unwrap_or_default();
  if matches!(first_op_code, OpCode::Text) && from_utf8_basic(message).is_err() {
    return Err(crate::Error::InvalidUTF8);
  }
  Ok(())
}

async fn fetch_frame_from_stream<D, SR, const IS_CLIENT: bool>(
  max_payload_len: usize,
  nc_rsv1: u8,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  stream_reader: &mut SR,
) -> crate::Result<ReadFrameInfo>
where
  D: WebSocketDecompression,
  SR: StreamReader,
{
  let rfi = ReadFrameInfo::from_stream::<D, _, IS_CLIENT>(
    max_payload_len,
    nc_rsv1,
    network_buffer,
    no_masking,
    stream_reader,
  )
  .await?;
  network_buffer.read_payload(rfi.payload_len, stream_reader).await?;
  Ok(rfi)
}

async fn fetch_frames_from_stream<D, SR, const IS_CLIENT: bool>(
  max_payload_len: usize,
  nc_rsv1: u8,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  reader_batches: &mut VectorU8<(ReadFrameInfo, Range<usize>)>,
  stream_reader: &mut SR,
) -> crate::Result<()>
where
  D: WebSocketDecompression,
  SR: StreamReader,
{
  let first_rfi = fetch_frame_from_stream::<D, _, IS_CLIENT>(
    max_payload_len,
    nc_rsv1,
    network_buffer,
    no_masking,
    stream_reader,
  )
  .await?;
  let first_end = network_buffer.current_end_idx();
  let first_begin = first_end.wrapping_sub(first_rfi.payload_len);
  let first_fin = first_rfi.fin;
  reader_batches.push((first_rfi, (first_begin..first_end).into()))?;
  if !first_fin {
    return Ok(());
  }
  let mut bytes = network_buffer.following();
  let mut offset = first_end;
  loop {
    let before_len = bytes.len();
    let Some(rfi) =
      ReadFrameInfo::from_bytes::<D, IS_CLIENT>(&mut bytes, max_payload_len, nc_rsv1, no_masking)?
    else {
      break;
    };
    let payload_begin = offset.wrapping_add(before_len.wrapping_sub(bytes.len()));
    let payload_end = payload_begin.wrapping_add(rfi.payload_len);
    let Some(remaining) = bytes.get(rfi.payload_len..) else {
      break;
    };
    reader_batches.push((rfi, (payload_begin..payload_end).into()))?;
    bytes = remaining;
    offset = payload_end;
  }
  Ok(())
}

async fn manage_first_finished_frame<
  'frame,
  'nb,
  'rbf,
  D,
  R,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'nb mut BufStreamReader,
  no_masking: bool,
  payload_origin: WebSocketPayloadOrigin,
  rfi: &ReadFrameInfo,
  rng: &mut R,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  stream_writer: &mut SW,
  user_buffer: &'rbf mut VectorUsize<u8>,
) -> crate::Result<FrameMut<'frame>>
where
  'nb: 'frame,
  'rbf: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SW: StreamWriter,
{
  let payload = if !D::IS_NOOP && rfi.should_decompress {
    let begin = user_buffer.len();
    copy_from_compressed_nb_to_rb1::<D, IS_CLIENT>(
      nc,
      network_buffer,
      no_masking,
      user_buffer,
      rfi,
      max_payload_len.saturating_add(begin),
    )?;
    user_buffer.as_slice_mut().get_mut(begin..).unwrap_or_default()
  } else {
    let current_mut = network_buffer.current_mut();
    unmask_nb::<IS_CLIENT>(rfi.mask, current_mut, no_masking)?;
    payload_origin.manage_payload(current_mut, user_buffer)?
  };
  let _is_control_frame = manage_auto_reply::<_, _, HAS_AUTO_REPLY, IS_CLIENT>(
    stream_writer,
    no_masking,
    rfi.op_code,
    payload,
    rng,
    stream_bridge,
    write_control_frame_cb,
  )
  .await?;
  manage_op_code_of_first_final_frame(rfi.op_code, payload)?;
  Ok(Frame::new(true, rfi.op_code, payload, nc_rsv1))
}

fn manage_first_unfinished_frame<'iicf, D, const IS_CLIENT: bool>(
  buffer: &mut VectorUsize<u8>,
  is_in_continuation_frame: &'iicf mut Option<IsInContinuationFrame>,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  rfi: &ReadFrameInfo,
) -> crate::Result<&'iicf mut IsInContinuationFrame>
where
  D: WebSocketDecompression,
{
  copy_from_arbitrary_nb_to_rb1::<IS_CLIENT>(rfi.mask, network_buffer, no_masking, buffer)?;
  let iuc = manage_op_code_of_first_continuation_frame(
    rfi.op_code,
    buffer,
    if !D::IS_NOOP && rfi.should_decompress {
      |_| Ok(None)
    } else {
      manage_text_of_first_continuation_frame
    },
  )?;
  Ok(is_in_continuation_frame.insert(IsInContinuationFrame {
    iuc,
    op_code: rfi.op_code,
    should_decompress: rfi.should_decompress,
    user_buffer_begin: 0,
    user_buffer_end: 0,
  }))
}

async fn read_continuation_frames<
  D,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  continuation_buffer: &mut VectorUsize<u8>,
  final_buffer: &mut VectorUsize<u8>,
  is_in_continuation_frame: &mut IsInContinuationFrame,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  closed_conn_cb: &mut impl FnMut(&mut S),
  reader_buffer_first_cb: &mut impl FnMut(
    OpCode,
    usize,
    &mut D,
    usize,
    &mut VectorUsize<u8>,
    &mut VectorUsize<u8>,
  ) -> crate::Result<()>,
  recurrent_text_cb: &mut impl FnMut(&[u8], &mut Option<PartialChar>) -> crate::Result<()>,
  stream_reader: &mut impl FnMut(&mut S) -> &mut SR,
  stream_writer: &mut impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<Option<OpCode>>
where
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  loop {
    let mut rfi = fetch_frame_from_stream::<D, _, IS_CLIENT>(
      max_payload_len,
      nc_rsv1,
      network_buffer,
      no_masking,
      stream_reader(stream),
    )
    .await?;
    rfi.should_decompress = is_in_continuation_frame.should_decompress;
    let payload = network_buffer.current_mut();
    unmask_nb::<IS_CLIENT>(rfi.mask, payload, no_masking)?;
    // Same ordering rules of the first frame. See `read_frame`.
    if !HAS_AUTO_REPLY && rfi.op_code.is_close() {
      cold_path();
      closed_conn_cb(stream);
    }
    let is_control_frame = manage_auto_reply::<_, _, HAS_AUTO_REPLY, IS_CLIENT>(
      stream_writer(stream),
      no_masking,
      rfi.op_code,
      payload,
      rng,
      stream_bridge,
      write_control_frame_cb,
    )
    .await?;
    if HAS_AUTO_REPLY && rfi.op_code.is_close() {
      cold_path();
      closed_conn_cb(stream);
    }
    if is_control_frame {
      return Ok(Some(rfi.op_code));
    }
    let message_len = if is_in_continuation_frame.should_decompress {
      continuation_buffer.len()
    } else {
      continuation_buffer.len().saturating_sub(is_in_continuation_frame.user_buffer_begin)
    };
    let accumulated_len = message_len.saturating_add(payload.len());
    if accumulated_len > max_payload_len {
      return Err(WebSocketError::VeryLargePayload.into());
    }
    continuation_buffer.extend_from_copyable_slice(payload)?;
    if !is_in_continuation_frame.should_decompress {
      is_in_continuation_frame.user_buffer_end = continuation_buffer.len();
    }
    if manage_op_code_of_continuation_frames(
      rfi.fin,
      is_in_continuation_frame.op_code,
      &mut is_in_continuation_frame.iuc,
      rfi.op_code,
      payload,
      recurrent_text_cb,
    )? {
      reader_buffer_first_cb(
        is_in_continuation_frame.op_code,
        max_payload_len,
        nc,
        is_in_continuation_frame.user_buffer_begin,
        continuation_buffer,
        final_buffer,
      )?;
      return Ok(None);
    }
  }
}

async fn read_frame_from_first_rfi<
  'frame,
  'nb,
  'rb,
  'ub,
  D,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  force_user_buffer: bool,
  is_in_continuation_frame_opt: &mut Option<IsInContinuationFrame>,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'nb mut BufStreamReader,
  no_masking: bool,
  payload_origin: WebSocketPayloadOrigin,
  reader_buffer: &'rb mut VectorUsize<u8>,
  first_rfi: ReadFrameInfo,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  user_buffer: &'ub mut VectorUsize<u8>,
  mut closed_conn_cb: impl FnMut(&mut S),
  mut stream_reader_cb: impl FnMut(&mut S) -> &mut SR,
  mut stream_writer_cb: impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<FrameMut<'frame>>
where
  'nb: 'frame,
  'rb: 'frame,
  'ub: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  if first_rfi.fin {
    if !HAS_AUTO_REPLY && first_rfi.op_code.is_close() {
      cold_path();
      closed_conn_cb(stream);
    }
    let rslt: Result<Frame<&mut [u8]>, crate::Error> =
      manage_first_finished_frame::<_, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
        max_payload_len,
        nc,
        nc_rsv1,
        network_buffer,
        no_masking,
        payload_origin,
        &first_rfi,
        rng,
        stream_bridge,
        stream_writer_cb(&mut *stream),
        user_buffer,
      )
      .await;
    if HAS_AUTO_REPLY && first_rfi.op_code.is_close() && rslt.is_ok() {
      cold_path();
      closed_conn_cb(stream);
    }
    return rslt;
  }
  let is_decompressing = !D::IS_NOOP && first_rfi.should_decompress;
  let user_buffer_begin = user_buffer.len();
  let buffer = if is_decompressing {
    reader_buffer.clear();
    &mut *reader_buffer
  } else {
    &mut *user_buffer
  };
  let is_in_continuation_frame = manage_first_unfinished_frame::<D, IS_CLIENT>(
    buffer,
    is_in_continuation_frame_opt,
    &mut *network_buffer,
    no_masking,
    &first_rfi,
  )?;
  is_in_continuation_frame.user_buffer_begin = user_buffer_begin;
  is_in_continuation_frame.user_buffer_end =
    if is_decompressing { user_buffer_begin } else { user_buffer.len() };
  let tuple = read_continuation_frames_until_final::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
    force_user_buffer,
    is_in_continuation_frame,
    max_payload_len,
    nc,
    nc_rsv1,
    network_buffer,
    no_masking,
    payload_origin,
    reader_buffer,
    rng,
    stream,
    stream_bridge,
    user_buffer,
    &mut closed_conn_cb,
    &mut stream_reader_cb,
    &mut stream_writer_cb,
  )
  .await?;
  let (frame, is_final) = tuple;
  if is_final {
    *is_in_continuation_frame_opt = None;
  }
  Ok(frame)
}

#[inline]
fn split_payload(bytes: &mut [u8], len: usize) -> (&mut [u8], &mut [u8]) {
  bytes.split_at_mut(len)
}

enum ReadFramesIter<'frame> {
  Many {
    batches: slice::Iter<'frame, (ReadFrameInfo, Range<usize>)>,
    offset: usize,
    remaining: &'frame mut [u8],
    rsv1: u8,
  },
  One(iter::Once<FrameMut<'frame>>),
}

impl<'frame> Iterator for ReadFramesIter<'frame> {
  type Item = FrameMut<'frame>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Many { batches, offset, remaining, rsv1 } => {
        let (rfi, range) = *batches.next()?;
        let payload_begin = range.start;
        let payload_end = range.end;
        let bytes = mem::take(remaining);
        let (_, tail0) = split_payload(bytes, payload_begin.wrapping_sub(*offset));
        let (payload, tail1) = split_payload(tail0, payload_end.wrapping_sub(payload_begin));
        *remaining = tail1;
        *offset = payload_end;
        Some(Frame::new(rfi.fin, rfi.op_code, payload, *rsv1))
      }
      Self::One(iter) => iter.next(),
    }
  }
}
