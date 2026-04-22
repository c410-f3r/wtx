use crate::{
  _SIMD_LEN,
  collection::{Clear, TryExtend, Vector},
  http2::{
    Http2Error, Http2ErrorCode,
    huffman_tables::{DECODE_TABLE, DECODED, ENCODE_TABLE, END_OF_STRING, ERROR},
    misc::protocol_err,
  },
  misc::{Lease, SingleTypeStorage},
};
use core::hint::cold_path;

const ENCODE_MASK: u64 = 0b1111_1111;

// Buffers are already pre-allocated in HPACK
pub(crate) fn huffman_decode<T>(from: &[u8], to: &mut T) -> crate::Result<()>
where
  T: Clear + Lease<[u8]> + SingleTypeStorage + TryExtend<[u8; 1]>,
{
  let mut curr_state = 0;
  let mut end_of_string = false;
  let mut has_error = false;
  let mut has_overflow = false;

  to.clear();

  'decode: {
    let (arrays, rem) = from.as_chunks::<{ _SIMD_LEN }>();
    for array in arrays {
      decode_all(&mut curr_state, &mut end_of_string, &mut has_error, &mut has_overflow, array, to);
      if has_error || has_overflow {
        break 'decode;
      }
    }
    decode_all(&mut curr_state, &mut end_of_string, &mut has_error, &mut has_overflow, rem, to);
  }

  if has_error {
    return Err(crate::Error::Http2ErrorGoAway(
      Http2ErrorCode::CompressionError,
      Http2Error::UnexpectedEndingHuffman,
    ));
  }

  if has_overflow {
    return Err(protocol_err(Http2Error::HpackDecodingBufferIsTooSmall));
  }

  let is_final = curr_state == 0 || end_of_string;
  if !is_final {
    return Err(crate::Error::Http2ErrorGoAway(
      Http2ErrorCode::CompressionError,
      Http2Error::UnexpectedEndingHuffman,
    ));
  }

  Ok(())
}

// Buffers are already pre-allocated in HPACK
pub(crate) fn huffman_encode(from: &[u8], to: &mut Vector<u8>) -> crate::Result<()> {
  let mut bits: u64 = 0;
  let mut bits_left: u64 = 40;
  let mut has_overflow = false;

  let (arrays, rem) = from.as_chunks::<{ _SIMD_LEN }>();
  for array in arrays {
    for elem in array {
      encode_all(*elem, &mut bits, &mut bits_left, &mut has_overflow, to);
    }
  }
  for elem in rem {
    encode_all(*elem, &mut bits, &mut bits_left, &mut has_overflow, to);
  }

  if has_overflow {
    return Err(protocol_err(Http2Error::HpackEncodingBufferIsTooSmall));
  }

  if bits_left != 40 {
    bits |= (1u64 << bits_left).wrapping_sub(1);
    if let Ok(n) = u8::try_from((bits >> 32) & ENCODE_MASK)
      && to.push(n).is_err()
    {
      return Err(protocol_err(Http2Error::HpackEncodingBufferIsTooSmall));
    }
  }
  Ok(())
}

#[inline(always)]
fn decode_4_bits(
  curr_state: &mut u8,
  end_of_string: &mut bool,
  has_error: &mut bool,
  input: u8,
) -> Option<u8> {
  if let Some((next_state, byte, flags)) = DECODE_TABLE
    .get(usize::from(*curr_state))
    .and_then(|slice_4bits| slice_4bits.get(usize::from(input)))
    .copied()
  {
    *has_error |= flags & ERROR == ERROR;
    let rslt = (flags & DECODED == DECODED).then_some(byte);
    *curr_state = next_state;
    *end_of_string = flags & END_OF_STRING == END_OF_STRING;
    rslt
  } else {
    cold_path();
    None
  }
}

#[inline(always)]
fn decode_all<T>(
  curr_state: &mut u8,
  end_of_string: &mut bool,
  has_error: &mut bool,
  has_overflow: &mut bool,
  slice: &[u8],
  to: &mut T,
) where
  T: Clear + Lease<[u8]> + SingleTypeStorage + TryExtend<[u8; 1]>,
{
  for elem in slice {
    let left_nibble = elem >> 4;
    if let Some(byte) = decode_4_bits(curr_state, end_of_string, has_error, left_nibble) {
      *has_overflow |= to.try_extend([byte]).is_err();
    }
    let right_nibble = elem & 0b0000_1111;
    if let Some(byte) = decode_4_bits(curr_state, end_of_string, has_error, right_nibble) {
      *has_overflow |= to.try_extend([byte]).is_err();
    }
  }
}

#[inline(always)]
fn encode_all(
  elem: u8,
  bits: &mut u64,
  bits_left: &mut u64,
  has_overflow: &mut bool,
  to: &mut Vector<u8>,
) {
  let Some((nbits, code)) = ENCODE_TABLE.get(usize::from(elem)).copied() else {
    cold_path();
    return;
  };
  let bits_offset = bits_left.wrapping_sub(u64::from(nbits));
  *bits |= code << bits_offset;
  *bits_left = bits_offset;
  if *bits_left <= 32 {
    push_encoded_byte(bits, bits_left, has_overflow, to);
  }
  if *bits_left <= 32 {
    push_encoded_byte(bits, bits_left, has_overflow, to);
  }
  if *bits_left <= 32 {
    push_encoded_byte(bits, bits_left, has_overflow, to);
  }
  if *bits_left <= 32 {
    push_encoded_byte(bits, bits_left, has_overflow, to);
  }
}

#[inline(always)]
fn push_encoded_byte(
  bits: &mut u64,
  bits_left: &mut u64,
  has_overflow: &mut bool,
  to: &mut Vector<u8>,
) {
  let Ok(n) = u8::try_from((*bits >> 32) & ENCODE_MASK) else {
    cold_path();
    return;
  };
  *has_overflow |= to.push(n).is_err();
  *bits <<= 8;
  *bits_left = bits_left.wrapping_add(8);
}

#[cfg(kani)]
mod kani {
  use crate::{
    collection::Vector,
    http::_HeaderValueBuffer,
    http2::huffman::{huffman_decode, huffman_encode},
  };

  #[kani::proof]
  fn encode_and_decode() {
    let data: Vector<u8> = kani::any();
    let mut encoded = Vector::with_capacity(data.len()).unwrap();
    huffman_encode(&data, &mut encoded).unwrap();
    let mut decoded = _HeaderValueBuffer::default();
    if huffman_decode(&encoded, &mut decoded).is_ok() {
      assert_eq!(data.as_ref(), decoded.as_ref());
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    collection::Vector,
    http2::huffman::{huffman_decode, huffman_encode},
  };

  #[test]
  fn decode_and_encode() {
    let mut decode = Vector::new();
    let mut encode = Vector::new();

    decode_and_encode_cmp((&mut decode, &mut encode), b"o", &[0b00111111]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"0", &[7]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"A", &[(0x21 << 2) + 3]);

    decode_and_encode_cmp((&mut decode, &mut encode), b"#", &[255, 160 + 15]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"$", &[255, 200 + 7]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"\x0a", &[255, 255, 255, 240 + 3]);

    decode_and_encode_cmp((&mut decode, &mut encode), b"!0", &[254, 1]);
    decode_and_encode_cmp((&mut decode, &mut encode), b" !", &[0b01010011, 0b11111000]);
  }

  fn decode_and_encode_cmp(
    (decode_buffer, encode_buffer): (&mut Vector<u8>, &mut Vector<u8>),
    bytes: &[u8],
    encoded: &[u8],
  ) {
    huffman_decode(encoded, decode_buffer).unwrap();
    assert_eq!(&**decode_buffer, bytes);

    huffman_encode(bytes, encode_buffer).unwrap();
    assert_eq!(&**encode_buffer, encoded);

    decode_buffer.clear();
    encode_buffer.clear();
  }
}
