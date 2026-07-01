#![expect(
  clippy::indexing_slicing,
  clippy::missing_asserts_for_indexing,
  reason = "constant trait support"
)]

use crate::misc::int_conv::u8usize;
use core::{
  fmt::{Display, Formatter},
  str,
};

const LOWER_HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
const UPPER_HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

/// Hex Encode Mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HexEncMode {
  /// Lower case characters ***WITH*** a `0x` prefix
  WithPrefixLower,
  /// Upper case characters ***WITH*** a `0x` prefix
  WithPrefixUpper,
  /// Lower case characters ***WITHOUT*** a `0x` prefix
  WithoutPrefixLower,
  /// Upper case characters ***WITHOUT*** a `0x` prefix
  WithoutPrefixUpper,
}

/// Errors of hexadecimal operations
#[derive(Clone, Copy, Debug)]
pub enum HexError {
  /// Provided buffer is too small
  InsufficientBuffer,
  /// Eip55 encoding only supports input data lesser or equal to 32 bytes
  #[cfg(feature = "sha3")]
  InvalidEip55Input,
  /// Provided element is not a valid hex character
  InvalidHexCharacter,
  /// Provided data is not multiple of two
  OddLen,
}

/// Auxiliary structure that will always output hexadecimal characters when displayed.
#[derive(Debug)]
pub struct HexDisplay<'bytes>(
  /// Bytes.
  pub &'bytes [u8],
  /// See [`HexEncMode`].
  ///
  /// Defaults to [`HexEncMode::WithoutPrefixLower`] if `None`.
  pub Option<HexEncMode>,
);

impl Display for HexDisplay<'_> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let actual_mode = actual_mode(self.1);
    let table = match actual_mode {
      HexEncMode::WithPrefixLower | HexEncMode::WithoutPrefixLower => LOWER_HEX_CHARS,
      HexEncMode::WithPrefixUpper | HexEncMode::WithoutPrefixUpper => UPPER_HEX_CHARS,
    };
    if matches!(actual_mode, HexEncMode::WithPrefixLower | HexEncMode::WithPrefixUpper) {
      write!(f, "0x")?;
    }
    for byte in self.0 {
      let (lhs, rhs) = byte_to_hex(*byte, table);
      write!(f, "{}{}", char::from(lhs), char::from(rhs))?;
    }
    Ok(())
  }
}

/// <https://eips.ethereum.org/EIPS/eip-55>
#[cfg(feature = "sha3")]
#[inline]
pub fn eip55_encode<'to>(from: &[u8], to: &'to mut [u8]) -> crate::Result<&'to str> {
  use sha3::Digest as _;
  if from.len() > 32 {
    return Err(HexError::InvalidEip55Input.into());
  }
  let rslt_len = hex_encode(from, Some(HexEncMode::WithPrefixLower), to)?.len();
  let Some([_, _, hex @ ..]) = to.get_mut(..rslt_len) else {
    return Ok("");
  };
  let hash: [u8; 32] = {
    let mut hasher = sha3::Keccak256::default();
    hasher.update(&*hex);
    hasher.finalize().into()
  };
  for (idx, byte) in hex.iter_mut().enumerate() {
    let is_letter = byte.is_ascii_lowercase();
    if !is_letter {
      continue;
    }
    let half_idx = hash.get(idx / 2).copied().unwrap_or_default();
    let nibble = if idx % 2 == 0 { half_idx >> 4 } else { half_idx & 0b0000_1111 };
    if nibble >= 8 {
      byte.make_ascii_uppercase();
    }
  }
  // SAFETY: HEX is always UTF-8
  unsafe { Ok(str::from_utf8_unchecked(to.get_mut(..rslt_len).unwrap_or_default())) }
}

/// Decodes `data` into `out` returning the affected part.
#[inline]
pub const fn hex_decode<'to>(
  mut from: &[u8],
  to: &'to mut [u8],
) -> Result<&'to mut [u8], HexError> {
  from = if let [b'0', b'x' | b'X', rest @ ..] = from { rest } else { from };
  let bytes_len = from.len() / 2;
  let Some((out_data, _)) = to.split_at_mut_checked(bytes_len) else {
    return Err(HexError::InsufficientBuffer);
  };
  let (arrays, rem) = from.as_chunks::<2>();
  if !rem.is_empty() {
    return Err(HexError::OddLen);
  }
  let mut idx = 0;
  while idx < bytes_len {
    let [lhs, rhs] = arrays[idx];
    out_data[idx] = match hex_to_bytes(lhs, rhs) {
      Ok(el) => el,
      Err(err) => return Err(err),
    };
    idx = idx.wrapping_add(1);
  }
  Ok(out_data)
}

/// Encodes `data` into `out` returning the affected part.
///
/// `mode` defaults to [`HexEncMode::WithoutPrefixLower`] if `None`.
#[inline]
pub const fn hex_encode<'to>(
  from: &[u8],
  hex_mode: Option<HexEncMode>,
  to: &'to mut [u8],
) -> Result<&'to str, HexError> {
  let actual_mode = actual_mode(hex_mode);
  let mut hex_len = from.len().wrapping_mul(2);
  let mut out_offset: usize = 0;
  match actual_mode {
    HexEncMode::WithPrefixLower | HexEncMode::WithPrefixUpper => {
      hex_len = hex_len.wrapping_add(2);
      if to.len() < hex_len {
        return Err(HexError::InsufficientBuffer);
      }
      to[0] = b'0';
      to[1] = b'x';
      out_offset = 2;
    }
    HexEncMode::WithoutPrefixLower | HexEncMode::WithoutPrefixUpper => {
      if to.len() < hex_len {
        return Err(HexError::InsufficientBuffer);
      }
    }
  }
  let table = match actual_mode {
    HexEncMode::WithPrefixLower | HexEncMode::WithoutPrefixLower => LOWER_HEX_CHARS,
    HexEncMode::WithPrefixUpper | HexEncMode::WithoutPrefixUpper => UPPER_HEX_CHARS,
  };
  let mut idx = 0;
  while idx < from.len() {
    let (b2, b3) = byte_to_hex(from[idx], table);
    let b0 = out_offset.wrapping_add(idx.wrapping_mul(2));
    let b1 = b0.wrapping_add(1);
    to[b0] = b2;
    to[b1] = b3;
    idx = idx.wrapping_add(1);
  }
  let Some((hex, _)) = to.split_at_checked(hex_len) else {
    return Ok("");
  };
  // SAFETY: HEX is always UTF-8
  unsafe { Ok(str::from_utf8_unchecked(hex)) }
}

const fn actual_mode(hem: Option<HexEncMode>) -> HexEncMode {
  if let Some(elem) = hem { elem } else { HexEncMode::WithoutPrefixLower }
}

#[expect(clippy::indexing_slicing, reason = "all bytes are limited to the array's length")]
const fn byte_to_hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
  let lhs_idx = u8usize(byte >> 4);
  let rhs_idx = u8usize(byte & 0b0000_1111);
  (table[lhs_idx], table[rhs_idx])
}

const fn hex_to_bytes(lhs: u8, rhs: u8) -> Result<u8, HexError> {
  const fn half(byte: u8) -> Result<u8, HexError> {
    match byte {
      b'0'..=b'9' => Ok(byte.wrapping_sub(b'0')),
      b'A'..=b'F' => Ok(byte.wrapping_sub(b'A').wrapping_add(10)),
      b'a'..=b'f' => Ok(byte.wrapping_sub(b'a').wrapping_add(10)),
      _ => Err(HexError::InvalidHexCharacter),
    }
  }
  let first = match half(lhs) {
    Ok(el) => el,
    Err(err) => return Err(err),
  };
  let second = match half(rhs) {
    Ok(el) => el,
    Err(err) => return Err(err),
  };
  Ok((first << 4) | second)
}

#[cfg(test)]
mod test {
  use crate::{
    codec::{HexDisplay, HexEncMode, hex_decode, hex_encode},
    collections::{ArrayVectorCopy, Vector},
  };

  #[test]
  fn decode_has_correct_output() {
    assert_eq!(hex_decode(b"61626364", &mut [0; 4]).unwrap(), b"abcd");
    assert_eq!(hex_decode(b"0x6162636465", &mut [0; 5]).unwrap(), b"abcde");
    assert!(hex_decode(b"6", &mut [0, 0, 0, 0]).is_err());
  }

  #[test]
  fn decode_insufficient_buffer() {
    assert!(hex_decode(b"61626364", &mut [0; 2]).is_err());
  }

  #[test]
  fn decode_invalid_character() {
    assert!(hex_decode(b"6G", &mut [0; 1]).is_err());
  }

  #[cfg(feature = "sha3")]
  #[test]
  fn eip55() {
    let mut buf = [0u8; 44];
    assert_eq!(
      crate::codec::eip55_encode(
        &[
          90, 174, 182, 5, 63, 62, 148, 201, 185, 160, 159, 51, 102, 148, 53, 231, 239, 27, 234,
          237,
        ],
        &mut buf
      )
      .unwrap(),
      "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
    );
  }

  #[test]
  fn encode_has_correct_output() {
    assert_eq!(hex_encode(&[], None, &mut [0u8; 8]).unwrap(), "");
    assert_eq!(hex_encode(b"AZ", None, &mut [0u8; 8]).unwrap(), "415a");
    assert_eq!(
      hex_encode(b"AZ", Some(HexEncMode::WithoutPrefixUpper), &mut [0u8; 8]).unwrap(),
      "415A"
    );
  }

  #[test]
  fn hex_display() {
    assert_eq!(
      &ArrayVectorCopy::<u8, 16>::try_from(format_args!(
        "{}",
        HexDisplay(b"abcdZ", Some(HexEncMode::WithoutPrefixLower))
      ))
      .unwrap(),
      "616263645a".as_bytes()
    );
    assert_eq!(
      &ArrayVectorCopy::<u8, 16>::try_from(format_args!(
        "{}",
        HexDisplay(b"abcdZ", Some(HexEncMode::WithPrefixLower))
      ))
      .unwrap(),
      "0x616263645a".as_bytes()
    );
  }

  #[test]
  fn roundtrip_various_lengths() {
    for len in 0u8..=20 {
      let data = Vector::from_iterator(0..len).unwrap();
      let mut enc_buf = Vector::from_iterator(0u8..len * 2 + 2).unwrap();
      let hex = hex_encode(&data, None, &mut enc_buf).unwrap();
      let mut dec_buf = Vector::from_iterator(0u8..len).unwrap();
      let decoded = hex_decode(hex.as_bytes(), &mut dec_buf).unwrap();
      assert_eq!(decoded, &data[..]);
    }
  }
}
