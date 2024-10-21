// |    Frame   |   With Decompression     | Without Decompression |
// |------------|--------------------------|-----------------------|
// |Single      |(NB -> RB1)¹              |(NB)¹                  |
// |Continuation|(NB -> RB1)* (RB1 -> RB2)¹|(NB -> RB2)*           |

use crate::{
  misc::{
    from_utf8_basic, ConnectionState, IncompleteUtf8Char, LeaseMut, PartitionedFilledBuffer,
    _read_until, from_utf8_ext, BufferParam, CompletionErr, ExtUtf8Error, FilledBuffer, FnMutFut,
    Rng, Stream,
  },
  web_socket::{
    close_payload, compression::NegotiatedCompression, misc::op_code, unmask::unmask,
    web_socket_writer::manage_normal_frame, CloseCode, Frame, FrameMut, OpCode, WebSocketError,
    MAX_CONTROL_PAYLOAD_LEN,
  },
};

const DECOMPRESSION_SUFFIX: [u8; 4] = [0, 0, 255, 255];

type ReadContinuationFramesCbs = (
  fn(&[u8]) -> crate::Result<Option<IncompleteUtf8Char>>,
  fn(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
  fn(&mut FilledBuffer, &mut PartitionedFilledBuffer, &ReadFrameInfo) -> crate::Result<()>,
);

#[inline]
pub(crate) async fn read_frame_from_stream<'nb, 'rb, 'rslt, NC, RNG, S, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  max_payload_len: usize,
  nc: &mut NC,
  network_buffer: &'nb mut PartitionedFilledBuffer,
  reader_buffer_first: &'rb mut FilledBuffer,
  reader_buffer_second: &'rb mut FilledBuffer,
  rng: &mut RNG,
  stream: &mut S,
) -> crate::Result<FrameMut<'rslt, IS_CLIENT>>
where
  'nb: 'rslt,
  'rb: 'rslt,
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
{
  #[inline]
  async fn fetch_frame_from_stream<NC, S, const IS_CLIENT: bool>(
    connection_state: &ConnectionState,
    max_payload_len: usize,
    nc: &NC,
    network_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<ReadFrameInfo>
  where
    NC: NegotiatedCompression,
    S: Stream,
  {
    let mut read = network_buffer._following_len();
    let rfi = fetch_header_from_stream::<_, _, IS_CLIENT>(
      max_payload_len,
      nc,
      network_buffer,
      &mut read,
      stream,
    )
    .await?;
    if connection_state.is_closed() && rfi.op_code != OpCode::Close {
      return Err(WebSocketError::ConnectionClosed.into());
    }
    let frame_len = rfi.payload_len.wrapping_add(rfi.header_len.into());
    fetch_payload_from_stream(frame_len, network_buffer, &mut read, stream).await?;
    network_buffer._set_indices(
      network_buffer._current_end_idx().wrapping_add(rfi.header_len.into()),
      rfi.payload_len,
      read.wrapping_sub(frame_len),
    )?;
    Ok(rfi)
  }

  #[inline]
  async fn fetch_header_from_stream<NC, S, const IS_CLIENT: bool>(
    max_payload_len: usize,
    nc: &NC,
    network_buffer: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<ReadFrameInfo>
  where
    NC: NegotiatedCompression,
    S: Stream,
  {
    let buffer = network_buffer._following_rest_mut();

    let first_two = _read_until::<2, S>(buffer, read, 0, stream).await?;

    let rsv1 = first_two[0] & 0b0100_0000;
    let rsv2 = first_two[0] & 0b0010_0000;
    let rsv3 = first_two[0] & 0b0001_0000;

    if rsv2 != 0 || rsv3 != 0 {
      return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
    }

    let should_decompress = if NC::IS_NOOP {
      false
    } else {
      if nc.rsv1() == 0 {
        if rsv1 != 0 {
          return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
        }
        false
      } else {
        rsv1 != 0
      }
    };

    let fin = first_two[0] & 0b1000_0000 != 0;
    let length_code = first_two[1] & 0b0111_1111;
    let op_code = op_code(first_two[0])?;

    let (mut header_len, payload_len) = match length_code {
      126 => {
        let payload_len = _read_until::<2, S>(buffer, read, 2, stream).await?;
        (4u8, u16::from_be_bytes(payload_len).into())
      }
      127 => {
        let payload_len = _read_until::<8, S>(buffer, read, 2, stream).await?;
        (10, u64::from_be_bytes(payload_len).try_into()?)
      }
      _ => (2, length_code.into()),
    };

    let mut mask = None;
    if !IS_CLIENT {
      mask = Some(_read_until::<4, S>(buffer, read, header_len.into(), stream).await?);
      header_len = header_len.wrapping_add(4);
    }

    if op_code.is_control() && !fin {
      return Err(WebSocketError::UnexpectedFragmentedControlFrame.into());
    }
    if op_code == OpCode::Ping && payload_len > MAX_CONTROL_PAYLOAD_LEN {
      return Err(WebSocketError::VeryLargeControlFrame.into());
    }
    if payload_len >= max_payload_len {
      return Err(WebSocketError::VeryLargePayload.into());
    }

    Ok(ReadFrameInfo { fin, header_len, mask, op_code, payload_len, should_decompress })
  }

  #[inline]
  async fn fetch_payload_from_stream<S>(
    frame_len: usize,
    network_buffer: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<()>
  where
    S: Stream,
  {
    network_buffer._reserve(frame_len)?;
    for _ in 0..=frame_len {
      if *read >= frame_len {
        return Ok(());
      }
      *read = read.wrapping_add(
        stream
          .read(network_buffer._following_rest_mut().get_mut(*read..).unwrap_or_default())
          .await?,
      );
    }
    Err(crate::Error::UnexpectedBufferState)
  }

  #[inline]
  async fn fetch_seq_frame_cb<NC, S, const IS_CLIENT: bool>(
    stream: &mut S,
    connection_state: &ConnectionState,
    max_payload_len: usize,
    nc: &NC,
    network_buffer: &mut PartitionedFilledBuffer,
  ) -> crate::Result<ReadFrameInfo>
  where
    NC: NegotiatedCompression,
    S: Stream,
  {
    fetch_frame_from_stream::<_, _, IS_CLIENT>(
      connection_state,
      max_payload_len,
      nc,
      network_buffer,
      stream,
    )
    .await
  }

  #[inline]
  async fn write_control_frame_cb<S>(
    stream: &mut S,
    header: &[u8],
    payload: &[u8],
  ) -> crate::Result<()>
  where
    S: Stream,
  {
    stream.write_all_vectored(&[header, payload]).await?;
    Ok(())
  }

  read_frame(
    stream,
    connection_state,
    max_payload_len,
    nc,
    network_buffer,
    reader_buffer_first,
    reader_buffer_second,
    rng,
    fetch_seq_frame_cb::<NC, S, IS_CLIENT>,
    write_control_frame_cb::<S>,
  )
  .await
}

// Intermediate compressed continuation frame
#[inline]
fn copy_from_compressed_nb_to_db_recurrent<const IS_CLIENT: bool>(
  network_buffer: &mut PartitionedFilledBuffer,
  reader_buffer_first: &mut FilledBuffer,
  rfi: &ReadFrameInfo,
) -> crate::Result<()> {
  unmask_nb::<IS_CLIENT>(network_buffer, rfi)?;
  reader_buffer_first._extend_from_slice(network_buffer._current())?;
  network_buffer._clear_if_following_is_empty();
  Ok(())
}

// Final compressed single frame
#[inline]
fn copy_from_compressed_nb_to_db_single<NC, const IS_CLIENT: bool>(
  nc: &mut NC,
  network_buffer: &mut PartitionedFilledBuffer,
  reader_buffer_first: &mut FilledBuffer,
  rfi: &ReadFrameInfo,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  unmask_nb::<IS_CLIENT>(network_buffer, rfi)?;
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
  let recurrent = (rfi.payload_len / 2).max(8);
  let initial = rfi.payload_len.wrapping_add(recurrent);
  let payload_len_rslt = nc.decompress(
    input,
    reader_buffer_first,
    |local_db| expand_db(initial, local_db, before),
    |local_db, written| expand_db(recurrent, local_db, before.wrapping_add(written)),
  );
  if has_following {
    if let [.., a, b, c, d] = input {
      *a = original[0];
      *b = original[1];
      *c = original[2];
      *d = original[3];
    }
  }
  network_buffer._clear_if_following_is_empty();
  let payload_len = payload_len_rslt?;
  reader_buffer_first._truncate(before.wrapping_add(payload_len));
  Ok(())
}

#[inline]
fn unmask_nb<const IS_CLIENT: bool>(
  network_buffer: &mut PartitionedFilledBuffer,
  rfi: &ReadFrameInfo,
) -> crate::Result<()> {
  if !IS_CLIENT {
    let current = network_buffer._current_mut();
    unmask(current, rfi.mask.ok_or(WebSocketError::MissingFrameMask)?);
  }
  Ok(())
}

#[inline]
fn expand_db<'db>(
  additional: usize,
  reader_buffer_first: &'db mut FilledBuffer,
  written: usize,
) -> crate::Result<&'db mut [u8]> {
  reader_buffer_first._expand(BufferParam::Additional(additional))?;
  Ok(reader_buffer_first.get_mut(written..).unwrap_or_default())
}

/// If this method returns `false`, then a `ping` frame was received and the caller should fetch
/// more external data in order to get the desired frame.
#[inline]
async fn manage_auto_reply<A, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
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
        return Ok(true);
      }
      match payload {
        [] => {}
        [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
        [a, b, rest @ ..] => {
          let _ = from_utf8_basic(rest)?;
          let is_not_allowed = !CloseCode::try_from(u16::from_be_bytes([*a, *b]))?.is_allowed();
          if is_not_allowed || rest.len() > MAX_CONTROL_PAYLOAD_LEN - 2 {
            let mut payload = [0; MAX_CONTROL_PAYLOAD_LEN];
            close_payload(CloseCode::Protocol, &mut payload, rest);
            write_control_frame::<_, _, _, IS_CLIENT>(
              aux,
              connection_state,
              &mut Frame::new_fin(OpCode::Close, payload),
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
        rng,
        write_control_frame_cb,
      )
      .await?;
      Ok(false)
    }
    OpCode::Continuation | OpCode::Binary | OpCode::Pong | OpCode::Text => Ok(true),
  }
}

#[inline]
fn manage_continuation_text(
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
    Err(ExtUtf8Error::Invalid { .. }) => {
      return Err(crate::Error::InvalidUTF8);
    }
    Ok(_) => {}
  }
  Ok(())
}

#[inline]
async fn read_continuation_frames<A, NC, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  first_rfi: &ReadFrameInfo,
  max_payload_len: usize,
  nc: &mut NC,
  network_buffer: &mut PartitionedFilledBuffer,
  reader_buffer_first: &mut FilledBuffer,
  rng: &mut RNG,
  (first_text_cb, continuation_cb, copy_cb): ReadContinuationFramesCbs,
  fetch_seq_frame_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any ConnectionState, usize, &'any NC, &'any mut PartitionedFilledBuffer),
    Result = crate::Result<ReadFrameInfo>,
  >,
  write_control_frame_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  RNG: Rng,
{
  let mut iuc = {
    let begin = reader_buffer_first.len();
    copy_cb(reader_buffer_first, network_buffer, first_rfi)?;
    let curr_payload = reader_buffer_first.get(begin..).unwrap_or_default();
    match first_rfi.op_code {
      OpCode::Binary => None,
      OpCode::Text => first_text_cb(curr_payload)?,
      OpCode::Close | OpCode::Continuation | OpCode::Ping | OpCode::Pong => {
        return Err(WebSocketError::UnexpectedMessageFrame.into());
      }
    }
  };
  'continuation_frames: loop {
    let (curr_payload, fin, op_code) = 'auto_reply: loop {
      let args = (&mut *aux, &*connection_state, max_payload_len, &*nc, &mut *network_buffer);
      let mut rfi = fetch_seq_frame_cb.call(args).await?;
      rfi.should_decompress = first_rfi.should_decompress;
      let begin = reader_buffer_first.len();
      copy_cb(reader_buffer_first, network_buffer, &rfi)?;
      let curr_payload = reader_buffer_first.get_mut(begin..).unwrap_or_default();
      let should_stop = manage_auto_reply::<_, _, IS_CLIENT>(
        aux,
        connection_state,
        rfi.op_code,
        curr_payload,
        rng,
        write_control_frame_cb,
      )
      .await?;
      if should_stop {
        break 'auto_reply (curr_payload, rfi.fin, rfi.op_code);
      }
      reader_buffer_first._truncate(begin);
    };
    match op_code {
      OpCode::Continuation => {
        continuation_cb(curr_payload, &mut iuc)?;
        if fin {
          break 'continuation_frames;
        }
      }
      OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong | OpCode::Text => {
        return Err(WebSocketError::UnexpectedMessageFrame.into());
      }
    }
  }
  Ok(())
}

/// Returns `None` if the frame is single, otherwise, returns the necessary information to
/// continue fetching from the stream.
//
// FIXME(polonius): Return `(ReadFrameInfo, Option<&mut [u8]>)`
#[inline]
async fn read_first_frame<A, NC, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  max_payload_len: usize,
  nc: &mut NC,
  network_buffer: &mut PartitionedFilledBuffer,
  reader_buffer_first: &mut FilledBuffer,
  rng: &mut RNG,
  fetch_seq_frame_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any ConnectionState, usize, &'any NC, &'any mut PartitionedFilledBuffer),
    Result = crate::Result<ReadFrameInfo>,
  >,
  write_control_frame_cb: &mut impl for<'any> FnMutFut<
    (&'any mut A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<ReadFrameInfo>
where
  NC: NegotiatedCompression,
  RNG: Rng,
{
  loop {
    let args = (&mut *aux, &*connection_state, max_payload_len, &*nc, &mut *network_buffer);
    let rfi = fetch_seq_frame_cb.call(args).await?;
    if !rfi.fin {
      return Ok(rfi);
    }
    let payload = if rfi.should_decompress {
      copy_from_compressed_nb_to_db_single::<NC, IS_CLIENT>(
        nc,
        network_buffer,
        reader_buffer_first,
        &rfi,
      )?;
      &mut *reader_buffer_first
    } else {
      unmask_nb::<IS_CLIENT>(network_buffer, &rfi)?;
      network_buffer._current_mut()
    };
    let should_stop = manage_auto_reply::<_, _, IS_CLIENT>(
      aux,
      connection_state,
      rfi.op_code,
      payload,
      rng,
      write_control_frame_cb,
    )
    .await?;
    if should_stop {
      match rfi.op_code {
        OpCode::Continuation => {
          return Err(WebSocketError::UnexpectedMessageFrame.into());
        }
        OpCode::Text => {
          let _str_validation = from_utf8_basic(payload)?;
        }
        OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong => {}
      }
      return Ok(rfi);
    }
  }
}

#[inline]
async fn read_frame<'nb, 'rb, 'rslt, A, NC, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  max_payload_len: usize,
  nc: &mut NC,
  network_buffer: &'nb mut PartitionedFilledBuffer,
  reader_buffer_first: &'rb mut FilledBuffer,
  reader_buffer_second: &'rb mut FilledBuffer,
  rng: &mut RNG,
  mut fetch_seq_frame_cb: impl for<'any> FnMutFut<
    (&'any mut A, &'any ConnectionState, usize, &'any NC, &'any mut PartitionedFilledBuffer),
    Result = crate::Result<ReadFrameInfo>,
  >,
  mut write_control_frame_cb: impl for<'any> FnMutFut<
    (&'any mut A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<FrameMut<'rslt, IS_CLIENT>>
where
  'nb: 'rslt,
  'rb: 'rslt,
  NC: NegotiatedCompression,
  RNG: Rng,
{
  reader_buffer_first._clear();
  reader_buffer_second._clear();
  let first_rfi = read_first_frame::<_, _, _, IS_CLIENT>(
    aux,
    connection_state,
    max_payload_len,
    nc,
    network_buffer,
    reader_buffer_first,
    rng,
    &mut fetch_seq_frame_cb,
    &mut write_control_frame_cb,
  )
  .await?;
  if first_rfi.fin {
    return Ok(Frame::new(
      true,
      first_rfi.op_code,
      if first_rfi.should_decompress { reader_buffer_first } else { network_buffer._current_mut() },
      nc.rsv1(),
    ));
  }
  if first_rfi.should_decompress {
    read_continuation_frames::<_, _, _, IS_CLIENT>(
      aux,
      connection_state,
      &first_rfi,
      max_payload_len,
      nc,
      network_buffer,
      reader_buffer_first,
      rng,
      (
        |_| Ok(None),
        |_, _| Ok(()),
        |local_db, local_nb, rfi| {
          copy_from_compressed_nb_to_db_recurrent::<IS_CLIENT>(local_nb, local_db, rfi)
        },
      ),
      &mut fetch_seq_frame_cb,
      &mut write_control_frame_cb,
    )
    .await?;
  reader_buffer_first._extend_from_slice(&DECOMPRESSION_SUFFIX)?;
  let additional = reader_buffer_first.len();
    let payload_len = nc.decompress(
      reader_buffer_first,
      reader_buffer_second,
      |local_buffer| expand_db(additional, local_buffer, 0),
      |local_buffer, written| expand_db(additional, local_buffer, written),
    )?;
    reader_buffer_second._truncate(payload_len);
    if matches!(first_rfi.op_code, OpCode::Text) && from_utf8_basic(reader_buffer_second).is_err() {
      return Err(crate::Error::InvalidUTF8);
    }
    Ok(Frame::new(true, first_rfi.op_code, reader_buffer_second, nc.rsv1()))
  } else {
    read_continuation_frames::<_, _, _, IS_CLIENT>(
      aux,
      connection_state,
      &first_rfi,
      max_payload_len,
      nc,
      network_buffer,
      reader_buffer_first,
      rng,
      (
        |curr_payload| {
          Ok(match from_utf8_ext(curr_payload) {
            Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
              Some(incomplete_ending_char)
            }
            Err(ExtUtf8Error::Invalid { .. }) => {
              return Err(crate::Error::InvalidUTF8);
            }
            Ok(_) => None,
          })
        },
        if matches!(first_rfi.op_code, OpCode::Binary) {
          |_, _| Ok(())
        } else {
          manage_continuation_text
        },
        |local_db, local_nb, rfi| {
          copy_from_compressed_nb_to_db_recurrent::<IS_CLIENT>(local_nb, local_db, rfi)
        },
      ),
      &mut fetch_seq_frame_cb,
      &mut write_control_frame_cb,
    )
    .await?;
    Ok(Frame::new(true, first_rfi.op_code, reader_buffer_first, nc.rsv1()))
  }
}

#[inline]
async fn write_control_frame<A, P, RNG, const IS_CLIENT: bool>(
  aux: &mut A,
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
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
  manage_normal_frame(connection_state, frame, rng)?;
  wsc_cb.call((aux, frame.header(), frame.payload().lease())).await?;
  Ok(())
}

/// Parameters of the frame read from a stream
#[derive(Debug)]
struct ReadFrameInfo {
  fin: bool,
  header_len: u8,
  mask: Option<[u8; 4]>,
  op_code: OpCode,
  payload_len: usize,
  should_decompress: bool,
}
