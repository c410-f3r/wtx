use crate::{
  collections::{Clear, SingleTypeStorage, TryExtend, Vector},
  http2::{
    Http2Error, Http2ErrorCode,
    huffman_tables::{DECODE_TABLE, DECODED, ENCODE_NBITS, ENCODE_TABLE, END_OF_STRING, ERROR},
    misc::protocol_err,
  },
  misc::Lease,
};
use core::{
  hint::{cold_path, unreachable_unchecked},
  mem::MaybeUninit,
};

// Buffers are already pre-allocated in HPACK
pub(crate) fn huffman_decode<T>(from: &[u8], to: &mut T) -> crate::Result<()>
where
  T: Clear + Lease<[u8]> + SingleTypeStorage + TryExtend<[u8; 1]>,
{
  to.clear();

  let mut curr_state = 0;
  let mut end_of_string = false;
  let mut has_error = false;
  let mut has_overflow = false;

  for elem in from {
    let left_nibble = elem >> 4;
    if let Some(byte) =
      decode_4_bits(&mut curr_state, &mut end_of_string, &mut has_error, left_nibble)
    {
      has_overflow |= to.try_extend([byte]).is_err();
    }
    let right_nibble = elem & 0b0000_1111;
    if let Some(byte) =
      decode_4_bits(&mut curr_state, &mut end_of_string, &mut has_error, right_nibble)
    {
      has_overflow |= to.try_extend([byte]).is_err();
    }
  }

  let is_not_final = curr_state != 0 && !end_of_string;
  if has_error || is_not_final {
    cold_path();
    return Err(crate::Error::Http2ErrorGoAway(
      Http2ErrorCode::CompressionError,
      Http2Error::UnexpectedEndingHuffman,
    ));
  }

  if has_overflow {
    cold_path();
    return Err(protocol_err(Http2Error::HpackDecodingBufferIsTooSmall));
  }

  Ok(())
}

pub(crate) fn huffman_encode(data: &[u8], to: &mut Vector<u8>) -> crate::Result<()> {
  let encoded_len = encoded_len(data);
  let original_len = to.len();
  #[cfg(not(test))]
  if encoded_len > data.len() {
    to.extend_from_copyable_slice(data)?;
    return Ok(());
  }
  to.reserve(encoded_len)?;
  let (_, allocated) = to.split_at_spare_mut();
  let data_len = data.len();
  let mut bits = 0u64;
  let mut bits_left = 40u64;
  let mut data_idx = 0;
  let mut encoded_idx = 0;
  while data_idx < data_len {
    let data_byte = data.get(data_idx).copied().unwrap_or_default();
    encode_data_byte(&mut bits, &mut bits_left, data_byte, &mut encoded_idx, allocated);
    data_idx = data_idx.wrapping_add(1);
  }
  encode_data_byte_last(&mut bits, bits_left, &mut encoded_idx, allocated);
  // SAFETY: `reserve` already ensured at least `encoded_len` allocated elements and all associated
  //         bytes were written.
  unsafe {
    to.set_len(original_len.wrapping_add(encoded_len));
  }
  Ok(())
}

#[inline(always)]
fn decode_4_bits(
  curr_state: &mut u8,
  end_of_string: &mut bool,
  has_error: &mut bool,
  nibble: u8,
) -> Option<u8> {
  if let Some((next_state, byte, flags)) = DECODE_TABLE
    .get(usize::from(*curr_state))
    .and_then(|slice| slice.get(usize::from(nibble & 0b0000_1111)))
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
fn encode_data_byte(
  bits: &mut u64,
  bits_left: &mut u64,
  data_byte: u8,
  encoded_idx: &mut usize,
  to: &mut [MaybeUninit<u8>],
) {
  let (nbits, code) = ENCODE_TABLE.get(usize::from(data_byte)).copied().unwrap_or_default();
  let bits_offset = bits_left.wrapping_sub(u64::from(nbits));
  *bits |= u64::from(code) << bits_offset;
  *bits_left = bits_offset;

  let num = u32::try_from((*bits >> 8) & 0xFFFF_FFFF).unwrap_or_default().to_be_bytes();
  let [b0, b1, b2, b3] = num;
  let (increment, offset) = match *bits_left {
    0..=8 => {
      let Some([b4, b5, b6, b7, ..]) = to.get_mut(*encoded_idx..) else {
        // SAFETY: `huffman_encode` already asserted that `to` can fill the entire encoded slice
        unsafe { unreachable_unchecked() }
      };
      let _ = b4.write(b0);
      let _ = b5.write(b1);
      let _ = b6.write(b2);
      let _ = b7.write(b3);
      (4, 32)
    }
    9..=16 => {
      let Some([b4, b5, b6, ..]) = to.get_mut(*encoded_idx..) else {
        // SAFETY: `huffman_encode` already asserted that `to` can fill the entire encoded slice
        unsafe { unreachable_unchecked() }
      };
      let _ = b4.write(b0);
      let _ = b5.write(b1);
      let _ = b6.write(b2);
      (3, 24)
    }
    17..=24 => {
      let Some([b4, b5, ..]) = to.get_mut(*encoded_idx..) else {
        // SAFETY: `huffman_encode` already asserted that `to` can fill the entire encoded slice
        unsafe { unreachable_unchecked() }
      };
      let _ = b4.write(b0);
      let _ = b5.write(b1);
      (2, 16)
    }
    25..=32 => {
      let Some([b4, ..]) = to.get_mut(*encoded_idx..) else {
        // SAFETY: `huffman_encode` already asserted that `to` can fill the entire encoded slice
        unsafe { unreachable_unchecked() }
      };
      let _ = b4.write(b0);
      (1, 8)
    }
    _ => return,
  };
  *bits <<= offset;
  *bits_left = bits_left.wrapping_add(offset);
  *encoded_idx = encoded_idx.wrapping_add(increment);
}

#[inline(always)]
fn encode_data_byte_last(
  bits: &mut u64,
  bits_left: u64,
  encoded_idx: &mut usize,
  to: &mut [MaybeUninit<u8>],
) {
  const U8_MASK: u64 = 0b1111_1111;
  if bits_left != 40 {
    *bits |= (1u64 << bits_left).wrapping_sub(1);
    let num = u8::try_from((*bits >> 32) & U8_MASK).unwrap_or_default();
    // SAFETY: `huffman_encode` already asserted that `to` can fill the entire encoded slice
    let to_elem = unsafe { to.get_mut(*encoded_idx).unwrap_unchecked() };
    let _ = to_elem.write(num);
    *encoded_idx = encoded_idx.wrapping_add(1);
  }
}

#[inline(always)]
fn encoded_len(data: &[u8]) -> usize {
  let mut encoded_len_bits: usize = 0;
  for elem in data {
    let nbits: usize = ENCODE_NBITS.get(usize::from(*elem)).copied().unwrap_or_default().into();
    encoded_len_bits = encoded_len_bits.wrapping_add(nbits);
  }
  encoded_len_bits.wrapping_add(7) / 8
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::{
    collections::Vector,
    http2::huffman::{huffman_decode, huffman_encode},
  };
  use core::hint::black_box;

  #[bench]
  fn encode_and_decode(b: &mut test::Bencher) {
    let original = "bePokBJU7N9MNRO8a9soYcM2PWlO2TL2NuqkWAjU91y3hwg07IJ3AsgQxKXeENvT7tguFVzKg6pCmA\
    ex46end1Ir5AMmXryC9IVCRGbevs7df2I6VKlryg9Cfd/RWFjwxzY4Mcz7Coc25fde+zuxOe/dhBISaieTtGJQ6Fg+6XdDINmcnLA\
    ioc8DyDdrGRjdhNfPzJuyuuAhOXjDj752i3I5XZCizaYl0NDZk1FKBegcRXCCPolqhy0GPNUqrJgmufGir7DOyN8h/ukLLujlvNfr\
    jPDOEv60mpWk7MOgFO96i10WKY2J2G7Gk/a5kxFIQ5EoKFR9M2jr2a1uDxYpu0O5PVLk3xW2QElom2qUxQdAWf8ciW6qKTLWBnyNw\
    HNqUsEEndCItl8xiG5MLbzLY6Q95Rbe/Yd/ta2zT1y4uA1hkKPleer9zNqHBQkSSEfJy8LSrisoPxmQcZwXYYWF1U6o+7v0wZnuKV\
    WLJuuMoeXXWb6PutpNuZBjtmdCHjA/hwTQFXuHdM2RKSydprtMmO80uuOX9STfB5O71mFHskQ7wwzj/T6FD7fzvxthBI7kGvz7m4V\
    j7XgNGeKqyEjwenpk1OWov9bWYJtJSHixp9v9jgjRWIMQ7DMDav/apFnLekjYZSQNv72AElS+xCbIZlY22v0hwm10R6ejpHyKvNB1\
    D6PNpxXXApWONnaclHczw3utrL9mkATP2oSbmLK7gZwT9onsNaa1ej8Jt+iLgv9BKfyLNOOcIypF2FZOyy1dKN0a9rLqsuJpefAPW\
    CrkgiCJDE5S+dL5yGWhAY/RRjkhnfNvQ+ST1NFFoDd/h54EU3XvPaTN9J4ozr9yE+Lffbdnnb3N9mMxHOLvVZLsq/GYxpS5+yox99\
    0YS/1WahgpTl89iM2Nhdbv7mwGrXD/X3dxSbSy48tJqxJx9uM7e9lK7fRqm5u/YcmiOGNxvla/e9S5VfQl+Mvk2idbGOYQktWRRbM\
    2YJ2eBAz52J8kkPj7TyVn5ljRhsXGK9jNcG82";
    let mut to = Vector::with_capacity(original.len()).unwrap();
    let mut from = Vector::with_capacity(original.len()).unwrap();
    b.iter(move || {
      black_box({
        huffman_encode(original.as_bytes(), &mut from).unwrap();
        huffman_decode(&from, &mut to).unwrap();
        assert_eq!(original.as_bytes(), to.as_slice());
        from.clear();
        to.clear();
      })
    });
  }
}

#[cfg(test)]
mod test {
  use crate::{
    collections::Vector,
    http2::huffman::{huffman_decode, huffman_encode},
  };

  #[test]
  fn decode_and_encode() {
    let mut decode = Vector::new();
    let mut encode = Vector::new();

    decode_and_encode_cmp((&mut decode, &mut encode), b"o", &[0b00111111]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"0", &[7]);
    decode_and_encode_cmp((&mut decode, &mut encode), b"A", &[(0x21 << 2) + 3]);

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
