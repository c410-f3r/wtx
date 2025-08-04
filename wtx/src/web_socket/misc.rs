use crate::{
  misc::{ConnectionState, FnMutFut, from_utf8_basic},
  rng::Rng,
  stream::StreamWriter,
  web_socket::{
    CloseCode, Frame, MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OP_CODE_MASK, OpCode,
    WebSocketError, web_socket_writer::manage_normal_frame,
  },
};

/// The first two bytes of `payload` are filled with `code`. Does nothing if `payload` is
/// less than 2 bytes.
#[inline]
pub fn fill_with_close_code(code: CloseCode, payload: &mut [u8]) {
  let [a, b, ..] = payload else {
    return;
  };
  let [c, d] = u16::from(code).to_be_bytes();
  *a = c;
  *b = d;
}

pub(crate) fn fill_header_from_params<const IS_CLIENT: bool>(
  fin: bool,
  header: &mut [u8; MAX_HEADER_LEN],
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> u8 {
  fn first_header_byte(fin: bool, op_code: OpCode, rsv1: u8) -> u8 {
    (u8::from(fin) << 7) | rsv1 | u8::from(op_code)
  }

  match payload_len {
    0..=125 => {
      let [a, b, ..] = header;
      *a = first_header_byte(fin, op_code, rsv1);
      *b = u8::try_from(payload_len).unwrap_or_default();
      2
    }
    126..=65535 => {
      let [len_c, len_d] = u16::try_from(payload_len).map(u16::to_be_bytes).unwrap_or_default();
      let [a, b, c, d, ..] = header;
      *a = first_header_byte(fin, op_code, rsv1);
      *b = 126;
      *c = len_c;
      *d = len_d;
      4
    }
    _ => {
      let len = u64::try_from(payload_len).map(u64::to_be_bytes).unwrap_or_default();
      let [len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j] = len;
      let [a, b, c, d, e, f, g, h, i, j, ..] = header;
      *a = first_header_byte(fin, op_code, rsv1);
      *b = 127;
      *c = len_c;
      *d = len_d;
      *e = len_e;
      *f = len_f;
      *g = len_g;
      *h = len_h;
      *i = len_i;
      *j = len_j;
      10
    }
  }
}

pub(crate) const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & MASK_MASK != 0
}

pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & OP_CODE_MASK)
}

#[inline]
pub(crate) async fn write_close_reply<'payload, A, RNG, const IS_CLIENT: bool>(
  aux: A,
  connection_state: &mut ConnectionState,
  no_masking: bool,
  payload: &'payload [u8],
  rng: &mut RNG,
  write_control_frame_cb: impl for<'any> FnMutFut<
    (A, &'any [u8], &'any [u8]),
    Result = crate::Result<()>,
  >,
) -> crate::Result<bool>
where
  RNG: Rng,
{
  fn modify_nothing_fn(_: &mut [u8]) {}
  fn modify_payload_fn(slice: &mut [u8]) {
    fill_with_close_code(CloseCode::Protocol, slice);
  }

  if connection_state.is_closed() {
    return Err(crate::Error::ClosedConnection);
  }
  *connection_state = ConnectionState::Closed;
  let modify_nothing: fn(_: &mut [u8]) = modify_nothing_fn;
  let modify_payload: fn(_: &mut [u8]) = modify_payload_fn;
  let (actual_payload, modify_payload_cb, rslt) = match payload {
    [] => (payload, modify_nothing, Ok(true)),
    [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
    [a, b, rest @ ..] => {
      let _str_validation = from_utf8_basic(rest)?;
      let close_code = CloseCode::try_from(u16::from_be_bytes([*a, *b]))?;
      if !close_code.is_allowed() || rest.len() > MAX_CONTROL_PAYLOAD_LEN - 2 {
        (
          payload.get(..MAX_CONTROL_PAYLOAD_LEN).unwrap_or_default(),
          modify_payload,
          Err(WebSocketError::InvalidCloseFrame.into()),
        )
      } else {
        (payload, modify_nothing, Ok(true))
      }
    }
  };
  write_control_frame::<_, _, IS_CLIENT>(
    aux,
    connection_state,
    no_masking,
    OpCode::Close,
    actual_payload,
    rng,
    modify_payload_cb,
    write_control_frame_cb,
  )
  .await?;
  rslt
}

pub(crate) async fn write_control_frame<A, RNG, const IS_CLIENT: bool>(
  aux: A,
  connection_state: &mut ConnectionState,
  no_masking: bool,
  op_code: OpCode,
  payload: &[u8],
  rng: &mut RNG,
  modify_payload_cb: fn(&mut [u8]),
  mut wsc_cb: impl for<'any> FnMutFut<(A, &'any [u8], &'any [u8]), Result = crate::Result<()>>,
) -> crate::Result<()>
where
  RNG: Rng,
{
  let len = payload.len().min(MAX_CONTROL_PAYLOAD_LEN);
  let mut array = [0; MAX_CONTROL_PAYLOAD_LEN];
  let slice = array.get_mut(..len).unwrap_or_default();
  slice.copy_from_slice(payload.get(..len).unwrap_or_default());
  modify_payload_cb(slice);
  let mut frame = Frame::<_, IS_CLIENT>::new_fin(op_code, slice);
  manage_normal_frame(connection_state, &mut frame, no_masking, rng);
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
