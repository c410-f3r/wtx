use crate::web_socket::{CloseCode, OpCode, MAX_HEADER_LEN_USIZE};
use core::ops::Range;

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
    u8::from(fin) << 7 | rsv1 | u8::from(op_code)
  }

  #[inline]
  fn manage_mask<const IS_CLIENT: bool, const N: u8>(
    second_byte: &mut u8,
    [a, b, c, d]: [&mut u8; 4],
  ) -> u8 {
    if IS_CLIENT {
      *second_byte &= 0b0111_1111;
      *a = 0;
      *b = 0;
      *c = 0;
      *d = 0;
      N.wrapping_add(4)
    } else {
      N
    }
  }

  match payload_len {
    0..=125 => {
      let [a, b, c, d, e, f, ..] = header;
      *a = first_header_byte(fin, op_code, rsv1);
      *b = u8::try_from(payload_len).unwrap_or_default();
      manage_mask::<IS_CLIENT, 2>(b, [c, d, e, f])
    }
    126..=0xFFFF => {
      let [len_c, len_d] = u16::try_from(payload_len).map(u16::to_be_bytes).unwrap_or_default();
      let [a, b, c, d, e, f, g, h, ..] = header;
      *a = first_header_byte(fin, op_code, rsv1);
      *b = 126;
      *c = len_c;
      *d = len_d;
      manage_mask::<IS_CLIENT, 4>(b, [e, f, g, h])
    }
    _ => {
      let len = u64::try_from(payload_len).map(u64::to_be_bytes).unwrap_or_default();
      let [len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j] = len;
      let [a, b, c, d, e, f, g, h, i, j, k, l, m, n] = header;
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
      manage_mask::<IS_CLIENT, 10>(b, [k, l, m, n])
    }
  }
}

#[inline]
pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & 0b0000_1111)
}

#[inline]
pub(crate) fn _trim_bytes(bytes: &[u8]) -> &[u8] {
  _trim_bytes_end(_trim_bytes_begin(bytes))
}

#[inline]
pub(crate) fn _truncated_slice<T>(slice: &[T], range: Range<usize>) -> &[T] {
  let start = range.start;
  let end = range.end.min(slice.len());
  slice.get(start..end).unwrap_or_default()
}

#[inline]
fn _trim_bytes_begin(mut bytes: &[u8]) -> &[u8] {
  while let [first, rest @ ..] = bytes {
    if first.is_ascii_whitespace() {
      bytes = rest;
    } else {
      break;
    }
  }
  bytes
}

#[inline]
fn _trim_bytes_end(mut bytes: &[u8]) -> &[u8] {
  while let [rest @ .., last] = bytes {
    if last.is_ascii_whitespace() {
      bytes = rest;
    } else {
      break;
    }
  }
  bytes
}
