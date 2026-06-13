use crate::{
  collection::ArrayVectorU8,
  misc::{ConnectionState, FnMutFut, from_utf8_basic},
  rng::Rng,
  stream::StreamWriter,
  web_socket::{
    CloseCode, Frame, MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OP_CODE_MASK, OpCode,
    WebSocketError, web_socket_writer::manage_normal_frame,
  },
};

/// Copies `frame_code` and `frame_payload` into `buffer`.
#[inline]
pub fn fill_buffer_with_close_frame(
  buffer: &mut [u8],
  frame_code: CloseCode,
  frame_payload: &[u8],
) -> crate::Result<()> {
  let rest = fill_buffer_with_close_code(buffer, frame_code);
  let Some(slice) = rest.and_then(|el| el.get_mut(..frame_payload.len())) else {
    return Err(WebSocketError::InvalidCloseFrameParams.into());
  };
  slice.copy_from_slice(frame_payload);
  Ok(())
}

/// The first two bytes of `buffer` are filled with `code`. Does nothing if `buffer` is
/// less than 2 bytes.
#[inline]
pub fn fill_buffer_with_close_code(buffer: &mut [u8], code: CloseCode) -> Option<&mut [u8]> {
  let [b1, b2, rest @ ..] = buffer else {
    return None;
  };
  let [b3, b4] = u16::from(code).to_be_bytes();
  *b1 = b3;
  *b2 = b4;
  Some(rest)
}

/// Returns `true` if `payload` is greater than the maximum allowed length.
#[inline]
pub(crate) fn check_read_close_frame(
  connection_state: &mut ConnectionState,
  payload: &[u8],
) -> crate::Result<bool> {
  if connection_state.is_closed() {
    return Err(crate::Error::ClosedWebSocketConnection);
  }
  *connection_state = ConnectionState::Closed;
  match payload {
    [] => Ok(false),
    [_] => Err(WebSocketError::InvalidCloseFrame.into()),
    [b1, b2, rest @ ..] => {
      let _str_validation = from_utf8_basic(rest)?;
      let close_code = CloseCode::try_from(u16::from_be_bytes([*b1, *b2]))?;
      if !close_code.is_allowed() || rest.len() > MAX_CONTROL_PAYLOAD_LEN - 2 {
        Ok(true)
      } else {
        Ok(false)
      }
    }
  }
}

pub(crate) fn control_frame_payload(data: &[u8]) -> ([u8; MAX_CONTROL_PAYLOAD_LEN], u8) {
  let len = data.len().min(MAX_CONTROL_PAYLOAD_LEN);
  let mut array = [0; MAX_CONTROL_PAYLOAD_LEN];
  let slice = array.get_mut(..len).unwrap_or_default();
  slice.copy_from_slice(data.get(..len).unwrap_or_default());
  (array, len.try_into().unwrap_or_default())
}

pub(crate) fn header_from_params(
  fin: bool,
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> ArrayVectorU8<u8, MAX_HEADER_LEN> {
  fn first_header_byte(fin: bool, op_code: OpCode, rsv1: u8) -> u8 {
    (u8::from(fin) << 7) | rsv1 | u8::from(op_code)
  }

  let mut header = ArrayVectorU8::from_array([0; MAX_HEADER_LEN]);
  match payload_len {
    0..=125 => {
      let _rslt = header.push(first_header_byte(fin, op_code, rsv1));
      let _rslt = header.push(u8::try_from(payload_len).unwrap_or_default());
    }
    126..=65_535 => {
      let [len_c, len_d] = u16::try_from(payload_len).map(u16::to_be_bytes).unwrap_or_default();
      let _rslt = header.push(first_header_byte(fin, op_code, rsv1));
      let _rslt = header.push(126);
      let _rslt = header.push(len_c);
      let _rslt = header.push(len_d);
    }
    _ => {
      let len = u64::try_from(payload_len).map(u64::to_be_bytes).unwrap_or_default();
      let [len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j] = len;
      let _rslt = header.push(first_header_byte(fin, op_code, rsv1));
      let _rslt = header.push(127);
      let _rslt = header.push(len_c);
      let _rslt = header.push(len_d);
      let _rslt = header.push(len_e);
      let _rslt = header.push(len_f);
      let _rslt = header.push(len_g);
      let _rslt = header.push(len_h);
      let _rslt = header.push(len_i);
      let _rslt = header.push(len_j);
    }
  }
  header
}

pub(crate) const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & MASK_MASK != 0
}

pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & OP_CODE_MASK)
}

pub(crate) async fn write_control_frame<A, RNG, const IS_CLIENT: bool>(
  aux: A,
  connection_state: &mut ConnectionState,
  no_masking: bool,
  op_code: OpCode,
  payload: &mut [u8],
  rng: &mut RNG,
  mut wsc_cb: impl for<'any> FnMutFut<(A, &'any [u8], &'any [u8]), Result = crate::Result<()>>,
) -> crate::Result<()>
where
  RNG: Rng,
{
  let mut frame = Frame::new(true, op_code, payload, 0);
  manage_normal_frame::<_, _, IS_CLIENT>(connection_state, &mut frame, no_masking, rng);
  wsc_cb.call((aux, frame.header(), frame.payload())).await?;
  Ok(())
}

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
