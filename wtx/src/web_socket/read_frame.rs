// * NB = Network Buffer
// * CB = Compression Buffer
// * UB = User Buffer
//
// |    Frame   |   With Decompression   | Without Decompression |
// |------------|------------------------|-----------------------|
// |Single      |(NB -> UB)¹             |(NB)¹                  |
// |Continuation|(NB -> CB)¹⁺ (CB -> UB)¹|(NB -> UB)¹⁺           |

use core::hint::cold_path;

use crate::{
  codec::DecompressionFlush,
  collections::{ArrayVectorCopy, Vector},
  futures::FnMutFut,
  misc::{ExtUtf8Error, PartialChar, from_utf8_basic, from_utf8_ext, process_utf8_stream},
  rng::Rng,
  stream::{BufStreamReader, StreamReader, StreamWriter},
  web_socket::{
    CloseCode, Frame, FrameMut, MAX_CONTROL_PAYLOAD_LEN, OpCode, WebSocketError,
    WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    misc::{manage_read_close_frame, write_control_frame, write_control_frame_cb},
    read_frame_info::ReadFrameInfo,
    unmask::unmask,
    web_socket_bridge::WebSocketBridge,
    web_socket_compression::WebSocketDecompression,
  },
};

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
    OpCode::Pong => Ok(true),
    OpCode::Continuation | OpCode::Binary | OpCode::Text => Ok(false),
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

pub(crate) async fn read_frame<
  'frame,
  'nb,
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
  reader_buffer: &mut Vector<u8>,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  user_buffer: &'ub mut Vector<u8>,
  mut closed_conn_cb: impl FnMut(&mut S),
  mut stream_reader_cb: impl FnMut(&mut S) -> &mut SR,
  mut stream_writer_cb: impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<FrameMut<'frame>>
where
  'nb: 'frame,
  'ub: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  let is_in_continuation_frame = if let Some(elem) = is_in_continuation_frame_opt {
    elem
  } else {
    user_buffer.clear();
    let first_rfi = fetch_frame_from_stream::<D, _, IS_CLIENT>(
      max_payload_len,
      nc_rsv1,
      network_buffer,
      no_masking,
      stream_reader_cb(&mut *stream),
    )
    .await?;
    if first_rfi.fin {
      if first_rfi.op_code.is_close() {
        cold_path();
        closed_conn_cb(stream);
      }
      let rslt = manage_first_finished_frame::<_, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
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
      return rslt;
    }
    let buffer = if !D::IS_NOOP && first_rfi.should_decompress {
      reader_buffer.clear();
      &mut *reader_buffer
    } else {
      &mut *user_buffer
    };
    manage_first_unfinished_frame::<D, IS_CLIENT>(
      buffer,
      is_in_continuation_frame_opt,
      &mut *network_buffer,
      no_masking,
      &first_rfi,
    )?
  };
  let control_frame = if !D::IS_NOOP && is_in_continuation_frame.should_decompress {
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
      &mut copy_from_compressed_rb1_to_rb2,
      &mut |_, _| Ok(()),
      &mut stream_reader_cb,
      &mut stream_writer_cb,
    )
    .await?
  } else {
    read_continuation_frames::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      user_buffer,
      &mut Vector::new(),
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      rng,
      stream,
      stream_bridge,
      &mut |_, _, _, _| Ok(()),
      &mut manage_text_of_recurrent_continuation_frames,
      &mut stream_reader_cb,
      &mut stream_writer_cb,
    )
    .await?
  };
  let (op_code, payload) = if let Some(op_code) = control_frame {
    if op_code.is_close() {
      cold_path();
      closed_conn_cb(stream);
    }
    (op_code, payload_origin.manage_payload(network_buffer.current_mut(), user_buffer)?)
  } else {
    reader_buffer.clear();
    let op_code = is_in_continuation_frame.op_code;
    *is_in_continuation_frame_opt = None;
    (op_code, user_buffer.as_slice_mut())
  };
  Ok(Frame::new(true, op_code, payload, nc_rsv1))
}

pub(crate) fn unmask_nb<const IS_CLIENT: bool>(
  mask: Option<[u8; 4]>,
  network_buffer: &mut [u8],
  no_masking: bool,
) -> crate::Result<()> {
  if !IS_CLIENT && !no_masking {
    unmask(network_buffer, mask.ok_or(WebSocketError::MissingFrameMask)?);
  }
  Ok(())
}

fn copy_from_arbitrary_nb_to_rb1<const IS_CLIENT: bool>(
  mask: Option<[u8; 4]>,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  reader_buffer_first: &mut Vector<u8>,
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
  reader_buffer: &mut Vector<u8>,
  rfi: &ReadFrameInfo,
) -> crate::Result<()>
where
  D: WebSocketDecompression,
{
  unmask_nb::<IS_CLIENT>(rfi.mask, network_buffer.current_mut(), no_masking)?;
  let _ = nc.decompress(DecompressionFlush::NoFlush, network_buffer.current(), reader_buffer)?;
  let _ = nc.decompress(DecompressionFlush::SyncFlush, &DECOMPRESSION_SUFFIX, reader_buffer)?;
  if !D::IS_NOOP && nc.no_context_takeover() {
    nc.reset();
  }
  Ok(())
}

/// All previously compressed continuation frames were concatenated into rb2. This is the
/// final decompression.
fn copy_from_compressed_rb1_to_rb2<D>(
  first_op_code: OpCode,
  nc: &mut D,
  reader_buffer_first: &mut Vector<u8>,
  reader_buffer_second: &mut Vector<u8>,
) -> crate::Result<()>
where
  D: WebSocketDecompression,
{
  reader_buffer_first.extend_from_copyable_slice(&DECOMPRESSION_SUFFIX)?;
  let payload_len =
    nc.decompress(DecompressionFlush::SyncFlush, reader_buffer_first, reader_buffer_second)?;
  reader_buffer_second.truncate(payload_len);
  if !D::IS_NOOP && nc.no_context_takeover() {
    nc.reset();
  }
  if matches!(first_op_code, OpCode::Text) && from_utf8_basic(reader_buffer_second).is_err() {
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
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &'nb mut BufStreamReader,
  no_masking: bool,
  payload_origin: WebSocketPayloadOrigin,
  rfi: &ReadFrameInfo,
  rng: &mut R,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  stream_writer: &mut SW,
  user_buffer: &'rbf mut Vector<u8>,
) -> crate::Result<FrameMut<'frame>>
where
  'nb: 'frame,
  'rbf: 'frame,
  D: WebSocketDecompression,
  R: Rng,
  SW: StreamWriter,
{
  let payload = if !D::IS_NOOP && rfi.should_decompress {
    copy_from_compressed_nb_to_rb1::<D, IS_CLIENT>(
      nc,
      network_buffer,
      no_masking,
      user_buffer,
      rfi,
    )?;
    user_buffer.as_slice_mut()
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
  buffer: &mut Vector<u8>,
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
  continuation_buffer: &mut Vector<u8>,
  final_buffer: &mut Vector<u8>,
  is_in_continuation_frame: &mut IsInContinuationFrame,
  max_payload_len: usize,
  nc: &mut D,
  nc_rsv1: u8,
  network_buffer: &mut BufStreamReader,
  no_masking: bool,
  rng: &mut R,
  stream: &mut S,
  stream_bridge: &WebSocketBridge<IS_CLIENT>,
  reader_buffer_first_cb: &mut impl FnMut(
    OpCode,
    &mut D,
    &mut Vector<u8>,
    &mut Vector<u8>,
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
    if is_control_frame {
      return Ok(Some(rfi.op_code));
    }
    continuation_buffer.extend_from_copyable_slice(payload)?;
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
        nc,
        continuation_buffer,
        final_buffer,
      )?;
      return Ok(None);
    }
  }
}
