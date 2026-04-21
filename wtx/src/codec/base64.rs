pub(crate) mod standard;
pub(crate) mod url;

use crate::codec::alphabet::{Alphabet as _, DecodeStep, EncodeStep};

/// Set of allowed characters
#[derive(Clone, Copy, Debug)]
pub enum Base64Alphabet {
  /// Standard Base64 alphabet ***WITH*** padding. Contains `+` and `/`.
  Standard,
  /// Standard Base64 alphabet ***WITHOUT*** padding. Contains `+` and `/`.
  StandardNoPad,
  /// URL and filename safe alphabet ***WITH*** padding. Contains `-` and `_`.
  Url,
  /// URL and filename safe alphabet ***WITHOUT*** padding. Contains `-` and `_`.
  UrlNoPad,
}

/// Base64 Error
#[derive(Debug)]
pub enum Base64Error {
  /// The encoding of a sequence of bytes would overflow its capacity.
  EncodingLengthOverflow,
  /// Generic decoding error
  InvalidDecoding,
  /// Base64 can not be represented with the provided length.
  InvalidLength,
  /// The provided output buffer is too small.
  SmallOutput,
}

/// Decodes `data` into `out` returning the affected part.
#[inline]
pub fn base64_decode<'to>(
  alphabet: Base64Alphabet,
  mut from: &[u8],
  to: &'to mut [u8],
) -> crate::Result<&'to mut [u8]> {
  let (_, decoder, _, pad_opt) = parts(alphabet);
  if let Some(pad) = pad_opt {
    if !from.len().is_multiple_of(4) {
      return Err(Base64Error::InvalidLength.into());
    }
    from = strip_padding(from, *pad);
  }

  let Some(len) = decoded_len_no_pad(from.len()) else {
    return Err(Base64Error::InvalidLength.into());
  };
  let Some(local_to) = to.get_mut(..len) else {
    return Err(Base64Error::SmallOutput.into());
  };

  let (from_chunks, from_chunks_rem) = from.as_chunks::<4>();
  let (to_chunks, to_chunks_mut) = local_to.as_chunks_mut::<3>();

  let mut err = 0;
  for (from_chunk, to_chunk) in from_chunks.iter().zip(to_chunks) {
    err |= decode_3bytes(decoder, from_chunk, to_chunk);
  }

  if !from_chunks_rem.is_empty() {
    let mut buffer_in = [b'A'; 4];
    let mut buffer_out = [0u8; 3];
    if let Some(elem) = buffer_in.get_mut(..from_chunks_rem.len()) {
      elem.copy_from_slice(from_chunks_rem);
    }
    err |= decode_3bytes(decoder, &buffer_in, &mut buffer_out);
    err |= match from_chunks_rem.len() {
      2 => u8::from((buffer_out[1] | buffer_out[2]) != 0),
      3 => u8::from(buffer_out[2] != 0),
      _ => 1,
    };
    if let Some(elem) = buffer_out.get(..to_chunks_mut.len()) {
      to_chunks_mut.copy_from_slice(elem);
    }
  }

  if err == 0 { Ok(local_to) } else { Err(Base64Error::InvalidDecoding.into()) }
}

/// Base64 Decoded Length - Upper Bound
///
/// Returns an estimation of the decoded size of `encoded_len`, which may not be accurate but is
/// safe as an upper bound when allocation buffers.
#[inline]
pub const fn base64_decoded_len_ub(encoded_len: usize) -> usize {
  let rem = encoded_len % 4;
  let chunks = encoded_len / 4;
  let surplus = if rem > 0 { 1 } else { 0 };
  chunks.wrapping_add(surplus).wrapping_mul(3)
}

/// Encodes `data` into `out` returning the affected part.
#[inline]
pub fn base64_encode<'to>(
  alphabet: Base64Alphabet,
  from: &[u8],
  to: &'to mut [u8],
) -> crate::Result<&'to str> {
  let (base, _, encoder, pad_opt) = parts(alphabet);
  let Some(len) = base64_encoded_len(from.len(), pad_opt.is_some()) else {
    return Err(Base64Error::EncodingLengthOverflow.into());
  };
  let Some(local_to) = to.get_mut(..len) else {
    return Err(Base64Error::SmallOutput.into());
  };
  let (from_chunks, from_chunks_rem) = from.as_chunks::<3>();
  let (to_chunks, to_chunks_rem) = local_to.as_chunks_mut::<4>();
  let mut to_chunks_iter = to_chunks.iter_mut();
  for (from_chunk, to_chunk) in from_chunks.iter().zip(to_chunks_iter.by_ref()) {
    encode_3bytes(*base, encoder, from_chunk, to_chunk);
  }
  if let Some(pad) = pad_opt
    && let Some(to_chunk) = to_chunks_iter.next()
  {
    let mut buffer = [0u8; 3];
    if let Some(elem) = buffer.get_mut(..from_chunks_rem.len()) {
      elem.copy_from_slice(from_chunks_rem);
    }
    encode_3bytes(*base, encoder, &buffer, to_chunk);
    let flag = from_chunks_rem.len() == 1;
    let mask = u8::from(flag).wrapping_sub(1);
    to_chunk[2] = (to_chunk[2] & mask) | (pad & !mask);
    to_chunk[3] = *pad;
  } else {
    let mut buffer_in = [0u8; 3];
    let mut buffer_out = [0u8; 4];
    if let Some(elem) = buffer_in.get_mut(..from_chunks_rem.len()) {
      elem.copy_from_slice(from_chunks_rem);
    }
    encode_3bytes(*base, encoder, &buffer_in, &mut buffer_out);
    if let Some(elem) = buffer_out.get(..to_chunks_rem.len()) {
      to_chunks_rem.copy_from_slice(elem);
    }
  }
  // SAFETY: Base64 bytes are ASCII.
  Ok(unsafe { core::str::from_utf8_unchecked(local_to) })
}

/// Calculates the encoded length of `decoded_len`. Returns `None` is case of an overflow.
#[inline]
pub const fn base64_encoded_len(decoded_len: usize, is_padded: bool) -> Option<usize> {
  let rem = decoded_len % 3;
  let chunks = decoded_len / 3;
  let Some(len) = chunks.checked_mul(4) else {
    return None;
  };
  if rem > 0 {
    if is_padded { len.checked_add(4) } else { len.checked_add(rem.wrapping_add(1)) }
  } else {
    Some(len)
  }
}

/// Returns `1` if `a == b`.
//
// This is a best effort. See <https://godbolt.org/z/svKzM449n>
#[inline(always)]
fn byte_cmp_ct(a: u8, b: u8) -> u8 {
  let diff = a ^ b;
  let (_, overflow) = diff.overflowing_sub(1);
  u8::from(overflow)
}

/// Return a non-zero value if the bytes of `from` are not valid Base64 characters.
///
/// `[00AAAAAA] [00BBBBBB] [00CCCCCC] [00DDDDDD]` becomes `[AAAAAABB] [BBBBCCCC] [CCDDDDDD]`.
#[inline(always)]
fn decode_3bytes(decoder: &'static [DecodeStep], from: &[u8; 4], to: &mut [u8; 3]) -> u8 {
  let a_i16 = decode_6bits(decoder, from[0]);
  let b_i16 = decode_6bits(decoder, from[1]);
  let c_i16 = decode_6bits(decoder, from[2]);
  let d_i16 = decode_6bits(decoder, from[3]);
  if (a_i16 | b_i16 | c_i16 | d_i16) < 0 {
    return 1;
  }
  let a = u8::try_from(a_i16).unwrap_or_default();
  let b = u8::try_from(b_i16).unwrap_or_default();
  let c = u8::try_from(c_i16).unwrap_or_default();
  let d = u8::try_from(d_i16).unwrap_or_default();
  to[0] = (a << 2) | (b >> 4);
  to[1] = (b << 4) | (c >> 2);
  to[2] = (c << 6) | d;
  0
}

/// Outputs `0b00xx_xxxx`.
#[inline(always)]
fn decode_6bits(decoder: &'static [DecodeStep], from: u8) -> i16 {
  /// Always returns a mask of value `0` or `255` (-1 in `i16` or 255 in `u8`)
  ///
  /// Returns `-1` if `from` is in the range delimited by `a` and `b`.
  #[inline(always)]
  fn is_in_range(a: u8, b: u8, from: u8) -> i16 {
    let begin = i16::from(a).wrapping_sub(1);
    let end = i16::from(b).wrapping_add(1);
    let value = begin.wrapping_sub(i16::from(from)) & i16::from(from).wrapping_sub(end);
    value >> 8
  }

  let mut rslt: i16 = -1;
  for step in decoder {
    let value = match step {
      DecodeStep::Range(range, offset) => {
        let value = is_in_range(*range.start(), *range.end(), from);
        value & i16::from(from).wrapping_add(*offset)
      }
      DecodeStep::Eq(value, offset) => is_in_range(*value, *value, from) & *offset,
    };
    rslt = rslt.wrapping_add(value);
  }
  rslt
}

#[inline(always)]
const fn decoded_len_no_pad(encoded_len: usize) -> Option<usize> {
  let chunks = encoded_len / 4;
  let rem = encoded_len % 4;
  let extra = match rem {
    0 => 0,
    2 => 1,
    3 => 2,
    _ => return None,
  };
  Some(chunks.wrapping_mul(3).wrapping_add(extra))
}

#[inline(always)]
fn encode_3bytes(base: u8, encoder: &'static [EncodeStep], from: &[u8; 3], to: &mut [u8; 4]) {
  let a = i16::from(from[0]);
  let b = i16::from(from[1]);
  let c = i16::from(from[2]);
  to[0] = encode_6bits(base, encoder, a >> 2);
  to[1] = encode_6bits(base, encoder, ((a << 4) | (b >> 4)) & 63);
  to[2] = encode_6bits(base, encoder, ((b << 2) | (c >> 6)) & 63);
  to[3] = encode_6bits(base, encoder, c & 63);
}

#[inline(always)]
fn encode_6bits(base: u8, encoder: &'static [EncodeStep], from: i16) -> u8 {
  let mut rslt = from.wrapping_add(base.into());
  for &step in encoder {
    let value = match step {
      EncodeStep::Apply(threshold, offset) => {
        (i16::from(threshold).wrapping_sub(rslt) >> 8) & offset
      }
      EncodeStep::Diff(threshold, offset) => {
        (i16::from(threshold).wrapping_sub(from) >> 8) & offset
      }
    };
    rslt = rslt.wrapping_add(value);
  }
  u8::try_from(rslt).unwrap_or_default()
}

#[inline(always)]
const fn parts(
  alphabet: Base64Alphabet,
) -> (&'static u8, &'static [DecodeStep], &'static [EncodeStep], &'static Option<u8>) {
  match alphabet {
    Base64Alphabet::Standard => (
      &standard::Standard::BASE,
      standard::Standard::DECODER,
      standard::Standard::ENCODER,
      &standard::Standard::PAD,
    ),
    Base64Alphabet::StandardNoPad => (
      &standard::StandardNoPad::BASE,
      standard::StandardNoPad::DECODER,
      standard::StandardNoPad::ENCODER,
      &standard::StandardNoPad::PAD,
    ),
    Base64Alphabet::Url => (&url::Url::BASE, url::Url::DECODER, url::Url::ENCODER, &url::Url::PAD),
    Base64Alphabet::UrlNoPad => {
      (&url::UrlNoPad::BASE, url::UrlNoPad::DECODER, url::UrlNoPad::ENCODER, &url::UrlNoPad::PAD)
    }
  }
}

#[inline(always)]
fn strip_padding(bytes: &[u8], pad: u8) -> &[u8] {
  match bytes {
    [rest @ .., a, b] => {
      let a_is_equal = byte_cmp_ct(*a, pad) == 1;
      let b_is_equal = byte_cmp_ct(*b, pad) == 1;
      match (a_is_equal, b_is_equal) {
        (true, true) => rest,
        (false, true) => {
          let idx = bytes.len().wrapping_sub(1);
          bytes.get(..idx).unwrap_or_default()
        }
        _ => bytes,
      }
    }
    [rest @ .., a] => {
      if byte_cmp_ct(*a, pad) == 1 {
        rest
      } else {
        bytes
      }
    }
    _ => bytes,
  }
}

#[inline(always)]
const fn u8i16(n: u8) -> i16 {
  n as i16
}

#[cfg(test)]
mod tests {
  use crate::codec::{Base64Alphabet, base64::base64_encoded_len, base64_decode, base64_encode};

  #[test]
  fn decode_has_correct_output() {
    assert_eq!(base64_decode(Base64Alphabet::Standard, b"TQ==", &mut [0u8; 8]).unwrap(), b"M");
    assert_eq!(base64_decode(Base64Alphabet::Standard, b"TWE=", &mut [0u8; 8]).unwrap(), b"Ma");
    assert!(base64_decode(Base64Alphabet::Standard, b"TQ", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::Standard, b"TQ=", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::Standard, b"T==E", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::Standard, b"T=E=", &mut [0u8; 8]).is_err());

    assert!(base64_decode(Base64Alphabet::StandardNoPad, b"TQ==", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::StandardNoPad, b"TWE=", &mut [0u8; 8]).is_err());
    assert_eq!(base64_decode(Base64Alphabet::StandardNoPad, b"TQ", &mut [0u8; 8]).unwrap(), b"M");
    assert!(base64_decode(Base64Alphabet::StandardNoPad, b"TQ=", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::StandardNoPad, b"T==E", &mut [0u8; 8]).is_err());
    assert!(base64_decode(Base64Alphabet::StandardNoPad, b"T=E=", &mut [0u8; 8]).is_err());

    assert_eq!(base64_decode(Base64Alphabet::StandardNoPad, b"TWE", &mut [0u8; 8]).unwrap(), b"Ma");
    assert_eq!(base64_decode(Base64Alphabet::StandardNoPad, b"TQ", &mut [0u8; 8]).unwrap(), b"M");
  }

  #[test]
  fn encoded_len_has_correct_output() {
    assert_eq!(base64_encoded_len(20, false).unwrap(), 27);
    assert_eq!(base64_encoded_len(190, true).unwrap(), 256);
  }

  #[test]
  fn roundtrip() {
    let value = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut decode_buffer = [0; 32];
    let mut encode_buffer = [0; 32];
    let encoded = base64_encode(Base64Alphabet::Standard, &value, &mut encode_buffer).unwrap();
    assert_eq!(
      base64_decode(Base64Alphabet::Standard, encoded.as_bytes(), &mut decode_buffer).unwrap(),
      value
    )
  }
}
