use crate::{
  collections::ArrayVectorCopy,
  futures::FnMutFut,
  misc::from_utf8_basic,
  rng::Rng,
  stream::StreamWriter,
  web_socket::{
    CloseCode, Frame, MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OP_CODE_MASK, OpCode,
    WebSocketError, write_frame::mask_frame,
  },
};

pub(crate) fn header_from_params(
  fin: bool,
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> ArrayVectorCopy<u8, MAX_HEADER_LEN> {
  fn first_header_byte(fin: bool, op_code: OpCode, rsv1: u8) -> u8 {
    (u8::from(fin) << 7) | rsv1 | u8::from(op_code)
  }

  let mut header = ArrayVectorCopy::new();
  match payload_len {
    0..=125 => {
      let _rslt = header.extend_from_copyable_slice(&[
        first_header_byte(fin, op_code, rsv1),
        u8::try_from(payload_len).unwrap_or_default(),
      ]);
    }
    126..=65_535 => {
      let [len_c, len_d] = u16::try_from(payload_len).map(u16::to_be_bytes).unwrap_or_default();
      let _rslt = header.extend_from_copyable_slice(&[
        first_header_byte(fin, op_code, rsv1),
        126,
        len_c,
        len_d,
      ]);
    }
    _ => {
      let len = u64::try_from(payload_len).map(u64::to_be_bytes).unwrap_or_default();
      let [len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j] = len;
      let _rslt = header.extend_from_copyable_slice(&[
        first_header_byte(fin, op_code, rsv1),
        127,
        len_c,
        len_d,
        len_e,
        len_f,
        len_g,
        len_h,
        len_i,
        len_j,
      ]);
    }
  }
  header
}

pub(crate) const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & MASK_MASK != 0
}

/// Returns `true` if `payload` is greater than the maximum allowed length.
#[inline]
pub(crate) fn manage_read_close_frame(
  close_code: CloseCode,
  payload: &mut [u8],
) -> crate::Result<bool> {
  match payload {
    [] => Ok(false),
    [_] => Err(WebSocketError::InvalidCloseFrame.into()),
    [b1, b2, rest @ ..] => {
      let _str_validation = from_utf8_basic(rest)?;
      let read_close_code = CloseCode::try_from(u16::from_be_bytes([*b1, *b2]))?;
      if !read_close_code.is_allowed() || rest.len() > MAX_CONTROL_PAYLOAD_LEN - 2 {
        let [b3, b4] = close_code.bytes();
        *b1 = b3;
        *b2 = b4;
        Ok(true)
      } else {
        Ok(false)
      }
    }
  }
}

pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & OP_CODE_MASK)
}

pub(crate) async fn write_control_frame<A, RNG, const IS_CLIENT: bool>(
  aux: A,
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
  mask_frame::<_, _, IS_CLIENT>(&mut frame, no_masking, rng);
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
