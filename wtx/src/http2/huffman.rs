use crate::{
  http2::huffman_tables::{DECODED, DECODE_TABLE, ENCODE_TABLE, ERROR, MAYBE_EOS},
  misc::{FilledBufferWriter, _unreachable},
};
use alloc::collections::VecDeque;

pub(crate) fn huffman_decode(from: &[u8], to: &mut VecDeque<u8>) -> crate::Result<()> {
  // Decodes 4 bits
  fn decode4(curr_state: &mut u16, input: u8, maybe_eos: &mut bool) -> crate::Result<Option<u8>> {
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
    let mut rslt = None;
    if flags & DECODED == DECODED {
      rslt = Some(byte);
    }
    *curr_state = next_state;
    *maybe_eos = flags & MAYBE_EOS == MAYBE_EOS;
    Ok(rslt)
  }

  let mut curr_state = 0;
  let mut maybe_eos = false;

  to.reserve(from.len() << 1);

  _iter4!(from, |elem| {
    let left_byte = elem >> 4;
    let right_byte = elem & 0b0000_1111;
    if let Some(byte) = decode4(&mut curr_state, left_byte, &mut maybe_eos)? {
      to.push_back(byte);
    }
    if let Some(byte) = decode4(&mut curr_state, right_byte, &mut maybe_eos)? {
      to.push_back(byte);
    }
  });

  if !(curr_state == 0 || maybe_eos) {
    return Err(crate::Error::UnexpectedEndingHuffman);
  }

  Ok(())
}

pub(crate) fn huffman_encode(from: &[u8], fbw: &mut FilledBufferWriter<'_>) {
  const M: u64 = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111;

  fn push_within_iter(bits: &mut u64, bits_left: &mut u64, fbw: &mut FilledBufferWriter<'_>) {
    let Ok(n) = u8::try_from((*bits >> 32) & M) else {
      _unreachable();
    };
    fbw._extend_from_byte(n);
    *bits <<= 8;
    *bits_left = bits_left.wrapping_add(8);
  }

  let mut bits: u64 = 0;
  let mut bits_left: u64 = 40;

  _iter4!(from, |elem| {
    let Some((nbits, code)) = ENCODE_TABLE.get(usize::from(*elem)).copied() else {
      _unreachable();
    };
    let bits_offset = bits_left.wrapping_sub(<_>::from(nbits));
    bits |= code << bits_offset;
    bits_left = bits_offset;
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, fbw);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, fbw);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, fbw);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, fbw);
    }
    if bits_left <= 32 {
      _unreachable()
    }
  });

  if bits_left != 40 {
    bits |= (1u64 << bits_left).wrapping_sub(1);
    let Ok(n) = u8::try_from((bits >> 32) & M) else {
      _unreachable();
    };
    fbw._extend_from_byte(n);
  }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::{http2::{huffman_decode, huffman_encode}, misc::FilledBufferWriter};
  use alloc::collections::VecDeque;

  #[bench]
  fn decode(b: &mut test::Bencher) {
    let mut data = crate::bench::_data(64 << 16);
    let mut dest = VecDeque::with_capacity(data.len());
    b.iter(|| huffman_decode(&mut data, &mut dest).unwrap());
  }

  #[bench]
  fn encode(b: &mut test::Bencher) {
    let mut data = crate::bench::_data(64 << 16);
    let mut dest = Vec::with_capacity(data.len());
    b.iter(|| huffman_encode(&mut data, &mut FilledBufferWriter::new(0, &mut dest)));
  }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod proptest {
  use crate::{http2::{huffman_decode, huffman_encode}, misc::FilledBufferWriter};
  use alloc::collections::VecDeque;

  #[test_strategy::proptest]
  fn encode_and_decode(data: Vec<u8>) {
    let mut encoded = Vec::with_capacity(data.len());
    huffman_encode(&data, &mut FilledBufferWriter::new(0, &mut encoded));
    let mut decoded = VecDeque::with_capacity(data.len());
    if huffman_decode(&encoded, &mut decoded).is_ok() {
      assert_eq!(&data, decoded.as_slices().0);
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{http2::huffman::{huffman_decode, huffman_encode}, misc::FilledBufferWriter};
  use alloc::collections::VecDeque;

  #[test]
  fn decode_and_encode() {
    let mut decode = VecDeque::new();
    let mut encode = Vec::new();

    decode_and_encode_cmp(&mut decode, &mut encode, &[0b00111111], b"o");
    decode_and_encode_cmp(&mut decode, &mut encode, &[7], b"0");
    decode_and_encode_cmp(&mut decode, &mut encode, &[(0x21 << 2) + 3], b"A");

    decode_and_encode_cmp(&mut decode, &mut encode, &[255, 160 + 15], b"#");
    decode_and_encode_cmp(&mut decode, &mut encode, &[255, 200 + 7], b"$");
    decode_and_encode_cmp(&mut decode, &mut encode, &[255, 255, 255, 240 + 3], b"\x0a");

    decode_and_encode_cmp(&mut decode, &mut encode, &[254, 1], b"!0");
    decode_and_encode_cmp(&mut decode, &mut encode, &[0b01010011, 0b11111000], b" !");
  }

  fn decode_cmp(decode: &mut VecDeque<u8>, bytes: &[u8], rslt: &[u8]) {
    huffman_decode(bytes, decode).unwrap();
    assert_eq!(decode.as_slices().0, rslt);
    decode.clear();
  }

  fn decode_and_encode_cmp(decode: &mut VecDeque<u8>, encode: &mut Vec<u8>, bytes: &[u8], rslt: &[u8]) {
    decode_cmp(decode, bytes, rslt);
    encode_cmp(encode, rslt, bytes);
  }

  fn encode_cmp(buf: &mut Vec<u8>, bytes: &[u8], rslt: &[u8]) {
    huffman_encode(bytes, &mut FilledBufferWriter::new(0, buf));
    assert_eq!(buf, rslt);
    buf.clear();
  }
}
