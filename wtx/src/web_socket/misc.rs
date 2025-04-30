use crate::web_socket::{CloseCode, MASK_MASK, MAX_HEADER_LEN_USIZE, OP_CODE_MASK, OpCode};

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

#[inline]
pub(crate) fn fill_header_from_params<const IS_CLIENT: bool>(
  fin: bool,
  header: &mut [u8; MAX_HEADER_LEN_USIZE],
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> u8 {
  #[inline]
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

#[inline]
pub(crate) const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & MASK_MASK != 0
}

#[inline]
pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & OP_CODE_MASK)
}
