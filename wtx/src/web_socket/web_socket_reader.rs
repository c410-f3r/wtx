// Common functions that used be used by pure WebSocket structures or tunneling protocols.
//
// * NB = Network Buffer
// * RCB = Compression Buffer
// * UB = User Buffer
//
//
// |    Frame    |   With Decompression     | Without Decompression |
// |-------------|--------------------------|-----------------------|
// |Single       |(NB -> UB)¹               |(NB)¹                  |
// |Continuation*|(NB -> RCB)¹⁺ (RCB -> UB)¹|(NB -> UB)¹⁺           |
//
// * Control frame payloads between continuation frames are located in NB

use crate::{
  collection::{ExpansionTy, IndexedStorageMut as _, Vector},
  misc::{
    CompletionErr, ConnectionState, ExtUtf8Error, FnMutFut, IncompleteUtf8Char, from_utf8_basic,
    from_utf8_ext,
    net::{PartitionedFilledBuffer, read_payload},
  },
  rng::Rng,
  stream::{StreamReader, StreamWriter},
  web_socket::{
    Frame, FrameMut, MAX_HEADER_LEN, OpCode, WebSocketError, WebSocketReadFrameTy,
    compression::NegotiatedCompression,
    is_in_continuation_frame::IsInContinuationFrame,
    misc::{write_close_reply, write_control_frame, write_control_frame_cb},
    read_frame_info::ReadFrameInfo,
    unmask::unmask,
  },
};

const DECOMPRESSION_SUFFIX: [u8; 4] = [0, 0, 255, 255];

/// Returns `true` if a control frame was received.
pub(crate) async fn manage_auto_reply<A, RNG, const IS_CLIENT: bool>(
  aux: A,
  connection_state: &mut ConnectionState,
  no_masking: bool,
  op_code: OpCode,
  payload: &mut [u8],
  rng: &mut RNG,
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
      write_close_reply::<_, _, IS_CLIENT>(
        aux,
        connection_state,
        no_masking,
        payload,
        rng,
        write_control_frame_cb,
      )
      .await
    }
    OpCode::Ping => {
      write_control_frame::<_, _, IS_CLIENT>(
        aux,
        connection_state,
        no_masking,
        OpCode::Pong,
        payload,
        rng,
        |_| {},
        write_control_frame_cb,
      )
      .await?;
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
  iuc: &mut Option<IncompleteUtf8Char>,
  op_code: OpCode,
  payload: &[u8],
  cb: &mut impl FnMut(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
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
  cb: fn(&[u8]) -> crate::Result<Option<IncompleteUtf8Char>>,
) -> crate::Result<Option<IncompleteUtf8Char>> {
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
) -> crate::Result<Option<IncompleteUtf8Char>> {
  Ok(match from_utf8_ext(payload) {
    Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => Some(incomplete_ending_char),
    Err(ExtUtf8Error::Invalid) => {
      return Err(crate::Error::InvalidUTF8);
    }
    Ok(_) => None,
  })
}

pub(crate) fn manage_text_of_recurrent_continuation_frames(
  curr_payload: &[u8],
  iuc: &mut Option<IncompleteUtf8Char>,
) -> crate::Result<()> {
  let tail = if let Some(mut incomplete) = iuc.take() {
    let (rslt, remaining) = incomplete.complete(curr_payload);
    match rslt {
      Err(CompletionErr::HasInvalidBytes) => {
        return Err(crate::Error::InvalidUTF8);
      }
      Err(CompletionErr::InsufficientInput) => {
        let _ = iuc.replace(incomplete);
        &[]
      }
      Ok(_) => remaining,
    }
  } else {
    curr_payload
  };
  match from_utf8_ext(tail) {
    Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
      *iuc = Some(incomplete_ending_char);
    }
    Err(ExtUtf8Error::Invalid) => {
      return Err(crate::Error::InvalidUTF8);
    }
    Ok(_) => {}
  }
  Ok(())
}

pub(crate) async fn read_frame<
  'frame,
  'nb,
  'ub,
  NC,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  connection_state: &mut ConnectionState,
  is_in_continuation_frame_opt: &mut Option<IsInContinuationFrame>,
  max_payload_len: usize,
  nc: &mut NC,
  nc_rsv1: u8,
  network_buffer: &'nb mut PartitionedFilledBuffer,
  no_masking: bool,
  reader_compression_buffer: &mut Vector<u8>,
  rng: &mut R,
  stream: &mut S,
  user_buffer: &'ub mut Vector<u8>,
  mut stream_reader: impl FnMut(&mut S) -> &mut SR,
  mut stream_writer: impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<(FrameMut<'frame, IS_CLIENT>, WebSocketReadFrameTy)>
where
  'nb: 'frame,
  'ub: 'frame,
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  let is_in_continuation_frame = if let Some(elem) = is_in_continuation_frame_opt {
    elem
  } else {
    user_buffer.clear();
    let first_rfi = fetch_frame_from_stream::<NC, _, IS_CLIENT>(
      max_payload_len,
      nc_rsv1,
      network_buffer,
      no_masking,
      stream_reader(&mut *stream),
    )
    .await?;
    if first_rfi.fin {
      return manage_first_finished_frame::<_, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
        connection_state,
        nc,
        nc_rsv1,
        network_buffer,
        no_masking,
        &first_rfi,
        rng,
        stream_writer(&mut *stream),
        user_buffer,
      )
      .await;
    }
    let buffer = if !NC::IS_NOOP && first_rfi.should_decompress {
      reader_compression_buffer.clear();
      &mut *reader_compression_buffer
    } else {
      &mut *user_buffer
    };
    manage_first_unfinished_frame::<NC, IS_CLIENT>(
      buffer,
      is_in_continuation_frame_opt,
      &mut *network_buffer,
      no_masking,
      &first_rfi,
    )?
  };
  let control_frame = if !NC::IS_NOOP && is_in_continuation_frame.should_decompress {
    read_continuation_frames::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      connection_state,
      &mut *reader_compression_buffer,
      &mut *user_buffer,
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      network_buffer,
      no_masking,
      rng,
      stream,
      &mut copy_from_compressed_rb1_to_rb2,
      &mut |_, _| Ok(()),
      &mut stream_reader,
      &mut stream_writer,
    )
    .await?
  } else {
    read_continuation_frames::<_, _, _, _, _, HAS_AUTO_REPLY, IS_CLIENT>(
      connection_state,
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
      &mut |_, _, _, _| Ok(()),
      &mut manage_text_of_recurrent_continuation_frames,
      &mut stream_reader,
      &mut stream_writer,
    )
    .await?
  };
  let (op_code, payload, wsrft) = if let Some(op_code) = control_frame {
    (op_code, network_buffer.current_mut(), WebSocketReadFrameTy::Internal)
  } else {
    reader_compression_buffer.clear();
    let op_code = is_in_continuation_frame.op_code;
    *is_in_continuation_frame_opt = None;
    (op_code, user_buffer.as_slice_mut(), WebSocketReadFrameTy::Provided)
  };
  Ok((Frame::new(true, op_code, payload, nc_rsv1), wsrft))
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
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  reader_buffer_first: &mut Vector<u8>,
) -> crate::Result<()> {
  let current_mut = network_buffer.current_mut();
  unmask_nb::<IS_CLIENT>(mask, current_mut, no_masking)?;
  reader_buffer_first.extend_from_copyable_slice(current_mut)?;
  Ok(())
}

fn copy_from_compressed_nb_to_rb1<NC, const IS_CLIENT: bool>(
  nc: &mut NC,
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  reader_buffer_first: &mut Vector<u8>,
  rfi: &ReadFrameInfo,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  unmask_nb::<IS_CLIENT>(rfi.mask, network_buffer.current_mut(), no_masking)?;
  network_buffer.reserve(4)?;
  let curr_end_idx = network_buffer.current().len();
  let curr_end_idx_p4 = curr_end_idx.wrapping_add(4);
  let has_following = network_buffer.has_following();
  let input = network_buffer.current_rest_mut().get_mut(..curr_end_idx_p4).unwrap_or_default();
  let original = if let [.., a, b, c, d] = input {
    let original = [*a, *b, *c, *d];
    *a = DECOMPRESSION_SUFFIX[0];
    *b = DECOMPRESSION_SUFFIX[1];
    *c = DECOMPRESSION_SUFFIX[2];
    *d = DECOMPRESSION_SUFFIX[3];
    original
  } else {
    [0, 0, 0, 0]
  };
  let before = reader_buffer_first.len();
  let additional = input.len().saturating_mul(2);
  let payload_len_rslt = nc.decompress(
    input,
    reader_buffer_first,
    |local_rb| expand_rb(additional, local_rb, before),
    |local_rb, written| expand_rb(additional, local_rb, before.wrapping_add(written)),
  );
  if has_following && let [.., a, b, c, d] = input {
    *a = original[0];
    *b = original[1];
    *c = original[2];
    *d = original[3];
  }
  let payload_len = payload_len_rslt?;
  reader_buffer_first.truncate(before.wrapping_add(payload_len));
  Ok(())
}

fn copy_from_compressed_rb1_to_rb2<NC>(
  first_op_code: OpCode,
  nc: &mut NC,
  reader_buffer_first: &mut Vector<u8>,
  reader_buffer_second: &mut Vector<u8>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  reader_buffer_first.extend_from_copyable_slice(&DECOMPRESSION_SUFFIX)?;
  let additional = reader_buffer_first.len().saturating_mul(2);
  let payload_len = nc.decompress(
    reader_buffer_first,
    reader_buffer_second,
    |local_rb| expand_rb(additional, local_rb, 0),
    |local_rb, written| expand_rb(additional, local_rb, written),
  )?;
  reader_buffer_second.truncate(payload_len);
  if matches!(first_op_code, OpCode::Text) && from_utf8_basic(reader_buffer_second).is_err() {
    return Err(crate::Error::InvalidUTF8);
  }
  Ok(())
}

fn expand_rb(
  additional: usize,
  reader_buffer_first: &mut Vector<u8>,
  written: usize,
) -> crate::Result<&mut [u8]> {
  reader_buffer_first.expand(ExpansionTy::Additional(additional), 0)?;
  Ok(reader_buffer_first.get_mut(written..).unwrap_or_default())
}

async fn fetch_frame_from_stream<NC, SR, const IS_CLIENT: bool>(
  max_payload_len: usize,
  nc_rsv1: u8,
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  stream_reader: &mut SR,
) -> crate::Result<ReadFrameInfo>
where
  NC: NegotiatedCompression,
  SR: StreamReader,
{
  network_buffer.clear_if_following_is_empty();
  network_buffer.reserve(MAX_HEADER_LEN)?;
  let mut read = network_buffer.following_len();
  let rfi = ReadFrameInfo::from_stream::<NC, _, IS_CLIENT>(
    max_payload_len,
    nc_rsv1,
    network_buffer,
    no_masking,
    &mut read,
    stream_reader,
  )
  .await?;
  let header_len = rfi.header_len.into();
  read_payload((header_len, rfi.payload_len), network_buffer, &mut read, stream_reader).await?;
  Ok(rfi)
}

async fn manage_first_finished_frame<
  'frame,
  'nb,
  'rbf,
  NC,
  R,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  connection_state: &mut ConnectionState,
  nc: &mut NC,
  nc_rsv1: u8,
  network_buffer: &'nb mut PartitionedFilledBuffer,
  no_masking: bool,
  rfi: &ReadFrameInfo,
  rng: &mut R,
  stream_writer: &mut SW,
  user_buffer: &'rbf mut Vector<u8>,
) -> crate::Result<(FrameMut<'frame, IS_CLIENT>, WebSocketReadFrameTy)>
where
  'nb: 'frame,
  'rbf: 'frame,
  NC: NegotiatedCompression,
  R: Rng,
  SW: StreamWriter,
{
  let (payload, wsrft) = if !NC::IS_NOOP && rfi.should_decompress {
    copy_from_compressed_nb_to_rb1::<NC, IS_CLIENT>(
      nc,
      network_buffer,
      no_masking,
      user_buffer,
      rfi,
    )?;
    (user_buffer.as_slice_mut(), WebSocketReadFrameTy::Provided)
  } else {
    let current_mut = network_buffer.current_mut();
    unmask_nb::<IS_CLIENT>(rfi.mask, current_mut, no_masking)?;
    (current_mut, WebSocketReadFrameTy::Internal)
  };
  if HAS_AUTO_REPLY {
    let _is_control_frame = manage_auto_reply::<_, _, IS_CLIENT>(
      stream_writer,
      connection_state,
      no_masking,
      rfi.op_code,
      payload,
      rng,
      write_control_frame_cb,
    )
    .await?;
  }
  manage_op_code_of_first_final_frame(rfi.op_code, payload)?;
  Ok((Frame::new(true, rfi.op_code, payload, nc_rsv1), wsrft))
}

fn manage_first_unfinished_frame<'iicf, NC, const IS_CLIENT: bool>(
  buffer: &mut Vector<u8>,
  is_in_continuation_frame: &'iicf mut Option<IsInContinuationFrame>,
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  rfi: &ReadFrameInfo,
) -> crate::Result<&'iicf mut IsInContinuationFrame>
where
  NC: NegotiatedCompression,
{
  copy_from_arbitrary_nb_to_rb1::<IS_CLIENT>(rfi.mask, network_buffer, no_masking, buffer)?;
  let iuc = manage_op_code_of_first_continuation_frame(
    rfi.op_code,
    buffer,
    if !NC::IS_NOOP && rfi.should_decompress {
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
  NC,
  R,
  S,
  SR,
  SW,
  const HAS_AUTO_REPLY: bool,
  const IS_CLIENT: bool,
>(
  connection_state: &mut ConnectionState,
  continuation_buffer: &mut Vector<u8>,
  final_buffer: &mut Vector<u8>,
  is_in_continuation_frame: &mut IsInContinuationFrame,
  max_payload_len: usize,
  nc: &mut NC,
  nc_rsv1: u8,
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  rng: &mut R,
  stream: &mut S,
  reader_buffer_first_cb: &mut impl FnMut(
    OpCode,
    &mut NC,
    &mut Vector<u8>,
    &mut Vector<u8>,
  ) -> crate::Result<()>,
  recurrent_text_cb: &mut impl FnMut(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
  stream_reader: &mut impl FnMut(&mut S) -> &mut SR,
  stream_writer: &mut impl FnMut(&mut S) -> &mut SW,
) -> crate::Result<Option<OpCode>>
where
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
{
  loop {
    let mut rfi = fetch_frame_from_stream::<NC, _, IS_CLIENT>(
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
    let is_control_frame = if HAS_AUTO_REPLY {
      manage_auto_reply::<_, _, IS_CLIENT>(
        stream_writer(stream),
        connection_state,
        no_masking,
        rfi.op_code,
        payload,
        rng,
        write_control_frame_cb,
      )
      .await?
    } else {
      rfi.op_code.is_control()
    };
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
      break;
    }
  }
  Ok(None)
}
