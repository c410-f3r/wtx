#![allow(clippy::unreachable)]

use crate::http2::huffman_tables::{DECODED, DECODE_TABLE, ENCODE_TABLE, ERROR, MAYBE_EOS};

#[doc = _internal_doc!()]
#[inline]
pub fn huffman_decode(from: &[u8], to: &mut Vec<u8>) -> crate::Result<()> {
  // Decodes 4 bits
  fn decode4(curr_state: &mut u16, input: u8, maybe_eos: &mut bool) -> crate::Result<Option<u8>> {
    let Some((next_state, byte, flags)) = DECODE_TABLE
      .get(usize::from(*curr_state))
      .and_then(|slice_4bits| slice_4bits.get(usize::from(input)))
      .copied()
    else {
      unreachable!();
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
      to.push(byte);
    }
    if let Some(byte) = decode4(&mut curr_state, right_byte, &mut maybe_eos)? {
      to.push(byte);
    }
  });

  if !(curr_state == 0 || maybe_eos) {
    return Err(crate::Error::UnexpectedEndingHuffman);
  }

  Ok(())
}

#[doc = _internal_doc!()]
#[inline]
pub fn huffman_encode(from: &[u8], to: &mut Vec<u8>) {
  const M: u64 = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111;

  fn push_within_iter(bits: &mut u64, bits_left: &mut u64, to: &mut Vec<u8>) {
    let Ok(n) = u8::try_from((*bits >> 32) & M) else {
      unreachable!();
    };
    to.push(n);
    *bits <<= 8;
    *bits_left = bits_left.wrapping_add(8);
  }

  let mut bits: u64 = 0;
  let mut bits_left: u64 = 40;

  to.reserve(from.len());

  _iter4!(from, |elem| {
    let Some((nbits, code)) = ENCODE_TABLE.get(usize::from(*elem)).copied() else {
      unreachable!();
    };
    let bits_offset = bits_left.wrapping_sub(<_>::from(nbits));
    bits |= code << bits_offset;
    bits_left = bits_offset;
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, to);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, to);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, to);
    }
    if bits_left <= 32 {
      push_within_iter(&mut bits, &mut bits_left, to);
    }
    if bits_left <= 32 {
      unreachable!()
    }
  });

  if bits_left != 40 {
    bits |= (1u64 << bits_left).wrapping_sub(1);
    let Ok(n) = u8::try_from((bits >> 32) & M) else {
      unreachable!();
    };
    to.push(n);
  }
}

#[cfg(test)]
mod test {
  use crate::http2::huffman::{huffman_decode, huffman_encode};

  #[cfg(feature = "_bench")]
  #[bench]
  fn bench_decode(b: &mut test::Bencher) {
    let mut data = crate::bench::_data(64 << 20);
    let mut dest = Vec::with_capacity(data.len());
    b.iter(|| huffman_encode(&mut data, &mut dest));
  }

  #[cfg(feature = "_bench")]
  #[bench]
  fn bench_encode(b: &mut test::Bencher) {
    let mut data = crate::bench::_data(64 << 20);
    let mut dest = Vec::with_capacity(data.len());
    b.iter(|| huffman_decode(&mut data, &mut dest));
  }

  #[test]
  fn decode_and_encode() {
    let mut buf = Vec::new();

    decode_and_encode_cmp(&mut buf, &[0b00111111], b"o");
    decode_and_encode_cmp(&mut buf, &[7], b"0");
    decode_and_encode_cmp(&mut buf, &[(0x21 << 2) + 3], b"A");

    decode_and_encode_cmp(&mut buf, &[255, 160 + 15], b"#");
    decode_and_encode_cmp(&mut buf, &[255, 200 + 7], b"$");
    decode_and_encode_cmp(&mut buf, &[255, 255, 255, 240 + 3], b"\x0a");

    decode_and_encode_cmp(&mut buf, &[254, 1], b"!0");
    decode_and_encode_cmp(&mut buf, &[0b01010011, 0b11111000], b" !");
  }

  #[cfg(feature = "_proptest")]
  #[test_strategy::proptest]
  fn proptest_encode_and_decode(data: Vec<u8>) {
    let mut encoded = Vec::with_capacity(data.len());
    huffman_encode(&data, &mut encoded);
    let mut decoded = Vec::with_capacity(data.len());
    if huffman_decode(&encoded, &mut decoded).is_ok() {
      assert_eq!(&data, &decoded);
    }
  }

  fn decode_cmp(buf: &mut Vec<u8>, bytes: &[u8], rslt: &[u8]) {
    huffman_decode(bytes, buf).unwrap();
    assert_eq!(buf.as_slice(), rslt);
    buf.clear();
  }

  fn decode_and_encode_cmp(buf: &mut Vec<u8>, bytes: &[u8], rslt: &[u8]) {
    decode_cmp(buf, bytes, rslt);
    encode_cmp(buf, rslt, bytes);
  }

  fn encode_cmp(buf: &mut Vec<u8>, bytes: &[u8], rslt: &[u8]) {
    huffman_encode(bytes, buf);
    assert_eq!(buf.as_slice(), rslt);
    buf.clear();
  }
}
