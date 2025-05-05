use crate::{
  collection::{ArrayVector, Vector},
  http2::{
    Http2Error, Http2ErrorCode,
    huffman_tables::{DECODE_TABLE, DECODED, ENCODE_TABLE, END_OF_STRING, ERROR},
    misc::protocol_err,
  },
  misc::{from_utf8_basic, hints::_unreachable},
};

pub(crate) fn huffman_decode<'to, const N: usize>(
  from: &[u8],
  to: &'to mut ArrayVector<u8, N>,
) -> crate::Result<&'to str> {
  fn decode_4_bits(
    curr_state: &mut u8,
    input: u8,
    end_of_string: &mut bool,
  ) -> crate::Result<Option<u8>> {
    let Some((next_state, byte, flags)) = DECODE_TABLE
      .get(usize::from(*curr_state))
      .and_then(|slice_4bits| slice_4bits.get(usize::from(input)))
      .copied()
    else {
      _unreachable();
    };
    if flags & ERROR == ERROR {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::CompressionError,
        Some(Http2Error::UnexpectedEndingHuffman),
      ));
    }
    let rslt = (flags & DECODED == DECODED).then_some(byte);
    *curr_state = next_state;
    *end_of_string = flags & END_OF_STRING == END_OF_STRING;
    Ok(rslt)
  }

  let mut curr_state = 0;
  let mut is_ok = true;
  let mut end_of_string = false;

  to.clear();

  _iter4!(
    from,
    {
      if !is_ok {
        break;
      }
    },
    |elem| {
      let left_nibble = elem >> 4;
      if let Some(byte) = decode_4_bits(&mut curr_state, left_nibble, &mut end_of_string)? {
        is_ok = to.push(byte).is_ok();
      }
      let right_nibble = elem & 0b0000_1111;
      if let Some(byte) = decode_4_bits(&mut curr_state, right_nibble, &mut end_of_string)? {
        is_ok = to.push(byte).is_ok();
      }
    }
  );

  if !is_ok {
    return Err(protocol_err(Http2Error::HpackDecodingBufferIsTooSmall));
  }

  let is_final = curr_state == 0 || end_of_string;
  if !is_final {
    return Err(crate::Error::Http2ErrorGoAway(
      Http2ErrorCode::CompressionError,
      Some(Http2Error::UnexpectedEndingHuffman),
    ));
  }

  Ok(from_utf8_basic(to)?)
}

pub(crate) fn huffman_encode(from: &[u8], wb: &mut Vector<u8>) -> crate::Result<()> {
  const MASK: u64 = 0b1111_1111;

  fn push_within_iter(
    bits: &mut u64,
    bits_left: &mut u64,
    wb: &mut Vector<u8>,
  ) -> crate::Result<()> {
    let Ok(n) = u8::try_from((*bits >> 32) & MASK) else {
      _unreachable();
    };
    wb.push(n)?;
    *bits <<= 8;
    *bits_left = bits_left.wrapping_add(8);
    Ok(())
  }

  let mut bits: u64 = 0;
  let mut bits_left: u64 = 40;

  wb.reserve((from.len() << 1).wrapping_add(5))?;

  _iter4!(from, {}, |elem| {
    let Some((nbits, code)) = ENCODE_TABLE.get(usize::from(*elem)).copied() else {
      _unreachable();
    };
    let bits_offset = bits_left.wrapping_sub(<_>::from(nbits));
    bits |= code << bits_offset;
    bits_left = bits_offset;
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb)?;
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb)?;
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb)?;
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb)?;
    }
    if bits_left <= 32 {
      _unreachable()
    }
    wb.reserve(5)?;
  });

  if bits_left != 40 {
    bits |= (1u64 << bits_left).wrapping_sub(1);
    let Ok(n) = u8::try_from((bits >> 32) & MASK) else {
      _unreachable();
    };
    wb.push(n)?;
  }
  Ok(())
}

#[cfg(kani)]
mod kani {
  use crate::{
    collection::Vector,
    http::_HeaderValueBuffer,
    http2::huffman::{huffman_decode, huffman_encode},
  };

  #[kani::proof]
  fn encode_and_decode(data: Vector<u8>) {
    let data = kani::any();
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
    http::_HeaderValueBuffer,
    http2::huffman::{huffman_decode, huffman_encode},
  };

  #[test]
  fn decode_and_encode() {
    let mut decode = _HeaderValueBuffer::default();
    let mut encode = Vector::default();

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
    (decode_buffer, encode_buffer): (&mut _HeaderValueBuffer, &mut Vector<u8>),
    bytes: &[u8],
    encoded: &[u8],
  ) {
    let _ = huffman_decode(encoded, decode_buffer).unwrap();
    assert_eq!(&**decode_buffer, bytes);

    huffman_encode(bytes, encode_buffer).unwrap();
    assert_eq!(&**encode_buffer, encoded);

    decode_buffer.clear();
    encode_buffer.clear();
  }
}
