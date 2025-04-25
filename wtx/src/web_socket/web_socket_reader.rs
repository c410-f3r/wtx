// Common functions that used be used by pure WebSocket structures or tunneling protocols.
//
// |    Frame   |   With Decompression     | Without Decompression |
// |------------|--------------------------|-----------------------|
// |Single      |(NB -> RB1)¹              |(NB)¹                  |
// |Continuation|(NB -> RB1)* (RB1 -> RB2)¹|(NB -> RB2)*           |

use crate::{
  misc::{
    BufferMode, CompletionErr, ConnectionState, ExtUtf8Error, FnMutFut, IncompleteUtf8Char,
    LeaseMut, Rng, StreamReader, StreamWriter, Vector, from_utf8_basic, from_utf8_ext,
    net::{PartitionedFilledBuffer, read_payload},
  },
  web_socket::{
    CloseCode, Frame, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN_USIZE, OpCode, WebSocketError,
    compression::NegotiatedCompression, fill_with_close_code, read_frame_info::ReadFrameInfo,
    unmask::unmask, web_socket_writer::manage_normal_frame,
  },
};

const DECOMPRESSION_SUFFIX: [u8; 4] = [0, 0, 255, 255];

#[inline]
pub(crate) fn copy_from_arbitrary_nb_to_rb1<const IS_CLIENT: bool>(
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  reader_buffer_first: &mut Vector<u8>,
  rfi: &ReadFrameInfo,
) -> crate::Result<()> {
  let current_mut = network_buffer._current_mut();
  unmask_nb::<IS_CLIENT>(current_mut, no_masking, rfi)?;
  reader_buffer_first.extend_from_copyable_slice(current_mut)?;
  Ok(())
}

#[inline]
pub(crate) fn copy_from_compressed_nb_to_rb1<NC, const IS_CLIENT: bool>(
  nc: &mut NC,
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  reader_buffer_first: &mut Vector<u8>,
  rfi: &ReadFrameInfo,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  unmask_nb::<IS_CLIENT>(network_buffer._current_mut(), no_masking, rfi)?;
  network_buffer._reserve(4)?;
  let curr_end_idx = network_buffer._current().len();
  let curr_end_idx_p4 = curr_end_idx.wrapping_add(4);
  let has_following = network_buffer._has_following();
  let input = network_buffer._current_rest_mut().get_mut(..curr_end_idx_p4).unwrap_or_default();
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
  if has_following {
    if let [.., a, b, c, d] = input {
      *a = original[0];
      *b = original[1];
      *c = original[2];
      *d = original[3];
    }
  }
  let payload_len = payload_len_rslt?;
  reader_buffer_first.truncate(before.wrapping_add(payload_len));
  Ok(())
}

#[inline]
pub(crate) fn copy_from_compressed_rb1_to_rb2<NC>(
  first_rfi: &ReadFrameInfo,
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
  if matches!(first_rfi.op_code, OpCode::Text) && from_utf8_basic(reader_buffer_second).is_err() {
    return Err(crate::Error::InvalidUTF8);
  }
  Ok(())
}

#[inline]
pub(crate) async fn fetch_frame_from_stream<SR, const IS_CLIENT: bool>(
  max_payload_len: usize,
  (nc_is_noop, nc_rsv1): (bool, u8),
  network_buffer: &mut PartitionedFilledBuffer,
  no_masking: bool,
  stream: &mut SR,
) -> crate::Result<ReadFrameInfo>
where
  SR: StreamReader,
{
  network_buffer._clear_if_following_is_empty();
  network_buffer._reserve(MAX_HEADER_LEN_USIZE)?;
  let mut read = network_buffer._following_len();
  let rfi = ReadFrameInfo::from_stream::<_, IS_CLIENT>(
    max_payload_len,
    (nc_is_noop, nc_rsv1),
    network_buffer,
    no_masking,
    &mut read,
    stream,
  )
  .await?;
  let header_len = rfi.header_len.into();
  read_payload((header_len, rfi.payload_len), network_buffer, &mut read, stream).await?;
  Ok(rfi)
}

/// If this method returns `false`, then a `ping` frame was received and the caller should fetch
/// more external data in order to get the desired frame.
#[inline]
pub(crate) async fn manage_auto_reply<A, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  no_masking: bool,
  op_code: OpCode,
  payload: &mut [u8],
  rng: &mut RNG,
  write_control_frame_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<bool>
where
  RNG: Rng,
{
  match op_code {
    OpCode::Close => {
      if connection_state.is_closed() {
        return Err(crate::Error::ClosedConnection);
      }
      *connection_state = ConnectionState::Closed;
      match payload {
        [] => {}
        [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
        [a, b, rest @ ..] => {
          let _ = from_utf8_basic(rest)?;
          let close_code = CloseCode::try_from(u16::from_be_bytes([*a, *b]))?;
          if !close_code.is_allowed() || rest.len() > MAX_CONTROL_PAYLOAD_LEN - 2 {
            fill_with_close_code(CloseCode::Protocol, payload);
            let payload_ret = payload.get_mut(..MAX_CONTROL_PAYLOAD_LEN).unwrap_or_default();
            write_control_frame::<_, _, _, IS_CLIENT>(
              aux,
              connection_state,
              &mut Frame::new_fin(OpCode::Close, payload_ret),
              no_masking,
              rng,
              write_control_frame_cb,
            )
            .await?;
            return Err(WebSocketError::InvalidCloseFrame.into());
          }
        }
      }
      write_control_frame::<_, _, _, IS_CLIENT>(
        aux,
        connection_state,
        &mut Frame::new_fin(OpCode::Close, payload),
        no_masking,
        rng,
        write_control_frame_cb,
      )
      .await?;
      Ok(true)
    }
    OpCode::Ping => {
      write_control_frame::<_, _, _, IS_CLIENT>(
        aux,
        connection_state,
        &mut Frame::new_fin(OpCode::Pong, payload),
        no_masking,
        rng,
        write_control_frame_cb,
      )
      .await?;
      Ok(false)
    }
    OpCode::Continuation | OpCode::Binary | OpCode::Pong | OpCode::Text => Ok(true),
  }
}

/// Returns `true` if `op_code` is a continuation frame and `fin` is also `true`.
#[inline]
pub(crate) fn manage_op_code_of_continuation_frames(
  fin: bool,
  first_op_code: OpCode,
  iuc: &mut Option<IncompleteUtf8Char>,
  op_code: OpCode,
  payload: &[u8],
  cb: fn(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
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
    OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong | OpCode::Text => {
      return Err(WebSocketError::UnexpectedFrame.into());
    }
  }
  Ok(false)
}

#[inline]
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

#[inline]
pub(crate) fn manage_op_code_of_first_final_frame(
  op_code: OpCode,
  payload: &[u8],
) -> crate::Result<()> {
  match op_code {
    OpCode::Close => {
      return Ok(());
    }
    OpCode::Continuation => {
      return Err(WebSocketError::UnexpectedFrame.into());
    }
    OpCode::Text => {
      let _str_validation = from_utf8_basic(payload)?;
    }
    OpCode::Binary | OpCode::Ping | OpCode::Pong => {}
  }
  Ok(())
}

#[inline]
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

#[inline]
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

#[inline]
pub(crate) fn unmask_nb<const IS_CLIENT: bool>(
  network_buffer: &mut [u8],
  no_masking: bool,
  rfi: &ReadFrameInfo,
) -> crate::Result<()> {
  if !IS_CLIENT && !no_masking {
    unmask(network_buffer, rfi.mask.ok_or(WebSocketError::MissingFrameMask)?);
  }
  Ok(())
}

#[inline]
pub(crate) async fn write_control_frame_cb<SW>(
  stream: &mut SW,
  header: &[u8],
  payload: &[u8],
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  stream.write_all_vectored(&[header, payload]).await?;
  Ok(())
}

#[inline]
fn expand_rb(
  additional: usize,
  reader_buffer_first: &mut Vector<u8>,
  written: usize,
) -> crate::Result<&mut [u8]> {
  reader_buffer_first.expand(BufferMode::Additional(additional), 0)?;
  Ok(reader_buffer_first.get_mut(written..).unwrap_or_default())
}

#[inline]
async fn write_control_frame<A, P, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
  no_masking: bool,
  rng: &mut RNG,
  wsc_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<()>
where
  P: LeaseMut<[u8]>,
  RNG: Rng,
{
  manage_normal_frame(connection_state, frame, no_masking, rng);
  wsc_cb.call((aux, frame.header(), frame.payload().lease())).await?;
  Ok(())
}
