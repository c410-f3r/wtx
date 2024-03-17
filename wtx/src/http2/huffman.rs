use crate::{
  http::MAX_HEADER_FIELD_LEN,
  http2::huffman_tables::{DECODED, DECODE_TABLE, ENCODE_TABLE, ERROR, MAYBE_EOS},
  misc::{ArrayVector, ByteVector, _unreachable},
};

pub(crate) fn huffman_decode(
  from: &[u8],
  wb: &mut ArrayVector<u8, MAX_HEADER_FIELD_LEN>,
) -> crate::Result<()> {
  fn decode_4_bits(
    curr_state: &mut u8,
    input: u8,
    maybe_eos: &mut bool,
  ) -> crate::Result<Option<u8>> {
    let Some((next_state, byte, flags)) = DECODE_TABLE
      .get(usize::from(*curr_state))
      .and_then(|slice_4bits| slice_4bits.get(usize::from(input)))
      .copied()
    else {
      _unreachable();
    };
    if flags & ERROR == ERROR {
      return Err(crate::Error::UnexpectedEndingHuffman);
    }
    let rslt = (flags & DECODED == DECODED).then_some(byte);
    *curr_state = next_state;
    *maybe_eos = flags & MAYBE_EOS == MAYBE_EOS;
    Ok(rslt)
  }

  let mut curr_state = 0;
  let mut is_ok = true;
  let mut maybe_eos = false;

  wb.clear();

  _iter4!(
    from,
    {
      if !is_ok {
        break;
      }
    },
    |elem| {
      let left_nibble = elem >> 4;
      if let Some(byte) = decode_4_bits(&mut curr_state, left_nibble, &mut maybe_eos)? {
        is_ok = wb.try_push(byte).is_ok();
      }
      let right_nibble = elem & 0b0000_1111;
      if let Some(byte) = decode_4_bits(&mut curr_state, right_nibble, &mut maybe_eos)? {
        is_ok = wb.try_push(byte).is_ok();
      }
    }
  );

  if !is_ok {
    return Err(crate::Error::HeaderFieldIsTooLarge);
  }

  if !(curr_state == 0 || maybe_eos) {
    return Err(crate::Error::UnexpectedEndingHuffman);
  }

  Ok(())
}

pub(crate) fn huffman_encode(from: &[u8], wb: &mut ByteVector) {
  const M: u64 = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111;

  fn push_within_iter(bits: &mut u64, bits_left: &mut u64, wb: &mut ByteVector) {
    let Ok(n) = u8::try_from((*bits >> 32) & M) else {
      _unreachable();
    };
    wb.push_within_cap(n);
    *bits <<= 8;
    *bits_left = bits_left.wrapping_add(8);
  }

  let mut bits: u64 = 0;
  let mut bits_left: u64 = 40;

  wb.reserve((from.len() << 1).wrapping_add(5));

  _iter4!(from, {}, |elem| {
    let Some((nbits, code)) = ENCODE_TABLE.get(usize::from(*elem)).copied() else {
      _unreachable();
    };
    let bits_offset = bits_left.wrapping_sub(<_>::from(nbits));
    bits |= code << bits_offset;
    bits_left = bits_offset;
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, wb);
    }
    if bits_left <= 32 {
      _unreachable()
    }
    wb.reserve(5);
  });

  if bits_left != 40 {
    bits |= (1u64 << bits_left).wrapping_sub(1);
    let Ok(n) = u8::try_from((bits >> 32) & M) else {
      _unreachable();
    };
    wb.push_within_cap(n);
  }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::{
    bench::_data,
    http::MAX_HEADER_FIELD_LEN,
    http2::{huffman_decode, huffman_encode},
    misc::{ArrayVector, Vector},
  };

  #[bench]
  fn decode(b: &mut test::Bencher) {
    let data = _data(64 << 16);
    let mut dest = ArrayVector::default();
    b.iter(|| {
      for chunk in data.chunks(MAX_HEADER_FIELD_LEN / 2) {
        std::dbg!(chunk);
        huffman_decode(chunk, &mut dest).unwrap();
        dest.clear();
      }
    });
  }

  #[bench]
  fn encode(b: &mut test::Bencher) {
    let mut data = _data(64 << 16);
    let mut dest = Vector::with_capacity(data.len());
    b.iter(|| huffman_encode(&mut data, &mut dest));
  }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod proptest {
  use crate::{
    http2::{huffman_decode, huffman_encode},
    misc::{ArrayVector, Vector},
  };
  use alloc::vec::Vec;

  #[test_strategy::proptest]
  fn encode_and_decode(data: Vec<u8>) {
    let mut encoded = Vector::with_capacity(data.len());
    huffman_encode(&data, &mut encoded);
    let mut decoded = ArrayVector::default();
    if huffman_decode(&encoded, &mut decoded).is_ok() {
      assert_eq!(&data, &*decoded);
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    http::MAX_HEADER_FIELD_LEN,
    http2::huffman::{huffman_decode, huffman_encode},
    misc::{ArrayVector, ByteVector, Vector},
  };

  #[test]
  fn decode_and_encode() {
    let mut decode = ArrayVector::default();
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
    (decode_buffer, encode_buffer): (&mut ArrayVector<u8, MAX_HEADER_FIELD_LEN>, &mut ByteVector),
    bytes: &[u8],
    encoded: &[u8],
  ) {
    huffman_decode(encoded, decode_buffer).unwrap();
    assert_eq!(&**decode_buffer, bytes);

    huffman_encode(&*bytes, encode_buffer);
    assert_eq!(&**encode_buffer, encoded);

    decode_buffer.clear();
    encode_buffer.clear();
  }
}
