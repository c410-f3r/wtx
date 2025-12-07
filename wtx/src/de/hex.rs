use core::fmt::{Display, Formatter};

const LOWER_HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
const UPPER_HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

/// Hex Encode Mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HexEncMode {
  /// Lower case characters ***with*** a `0x` prefix
  WithPrefixLower,
  /// Upper case characters ***with*** a `0x` prefix
  WithPrefixUpper,
  /// Lower case characters ***without*** a `0x` prefix
  WithoutPrefixLower,
  /// Upper case characters ***without*** a `0x` prefix
  WithoutPrefixUpper,
}

/// Errors of hexadecimal operations
#[derive(Debug)]
pub enum HexError {
  /// Provided buffer is too small
  InsufficientBuffer,
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
  /// Defaults to [`HexEncMode::WithPrefixLower`] if `None`.
  pub Option<HexEncMode>,
);

impl Display for HexDisplay<'_> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let actual_mode = actual_mode(self.1);
    if matches!(actual_mode, HexEncMode::WithPrefixLower | HexEncMode::WithPrefixUpper) {
      write!(f, "0x")?;
    }
    let table = match actual_mode {
      HexEncMode::WithPrefixLower | HexEncMode::WithoutPrefixLower => LOWER_HEX_CHARS,
      HexEncMode::WithPrefixUpper | HexEncMode::WithoutPrefixUpper => UPPER_HEX_CHARS,
    };
    for byte in self.0 {
      let (lhs, rhs) = byte_to_hex(*byte, table);
      write!(f, "{}{}", char::from(lhs), char::from(rhs))?;
    }
    Ok(())
  }
}

/// Decodes `data` into `out` returning the affected part.
#[inline]
pub fn decode_hex<'out>(mut data: &[u8], out: &'out mut [u8]) -> crate::Result<&'out mut [u8]> {
  data = if let [b'0', b'x', rest @ ..] = data { rest } else { data };
  let bytes_len = data.len() / 2;
  let Some(actual_out) = out.get_mut(..bytes_len) else {
    return Err(HexError::InsufficientBuffer.into());
  };
  let (arrays, rem) = data.as_chunks::<2>();
  if !rem.is_empty() {
    return Err(HexError::OddLen.into());
  }
  for ([a, b], byte) in arrays.iter().zip(&mut *actual_out) {
    *byte = hex_to_bytes(*a, *b)?;
  }
  Ok(actual_out)
}

/// Encodes `data` into `out` returning the affected part.
///
/// `mode` defaults to [`HexEncMode::WithPrefixLower`] if `None`.
#[inline]
pub fn encode_hex<'out>(
  data: &[u8],
  mode: Option<HexEncMode>,
  out: &'out mut [u8],
) -> crate::Result<&'out str> {
  let actual_mode = actual_mode(mode);
  let mut hex_len = data.len().wrapping_mul(2);
  let actual_out = match actual_mode {
    HexEncMode::WithPrefixLower | HexEncMode::WithPrefixUpper => {
      hex_len = hex_len.wrapping_add(2);
      let Some([a, b, actual_out @ ..]) = out.get_mut(..hex_len) else {
        return Err(HexError::InsufficientBuffer.into());
      };
      *a = b'0';
      *b = b'x';
      actual_out
    }
    HexEncMode::WithoutPrefixLower | HexEncMode::WithoutPrefixUpper => {
      let Some((actual_out, _)) = out.split_at_mut_checked(hex_len) else {
        return Err(HexError::InsufficientBuffer.into());
      };
      actual_out
    }
  };
  let (arrays, _) = actual_out.as_chunks_mut::<2>();
  let table = match actual_mode {
    HexEncMode::WithPrefixLower | HexEncMode::WithoutPrefixLower => LOWER_HEX_CHARS,
    HexEncMode::WithPrefixUpper | HexEncMode::WithoutPrefixUpper => UPPER_HEX_CHARS,
  };
  for (byte, [a, b]) in data.iter().zip(arrays) {
    let (lhs, rhs) = byte_to_hex(*byte, table);
    *a = lhs;
    *b = rhs;
  }
  // SAFETY: HEX is always UTF-8
  unsafe { Ok(str::from_utf8_unchecked(out.get_mut(..hex_len).unwrap_or_default())) }
}

const fn actual_mode(hem: Option<HexEncMode>) -> HexEncMode {
  if let Some(elem) = hem { elem } else { HexEncMode::WithPrefixLower }
}

#[expect(clippy::indexing_slicing, reason = "all bytes are limited to the array's length")]
fn byte_to_hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
  let lhs_idx: usize = (byte >> 4).into();
  let rhs_idx: usize = (byte & 0b0000_1111).into();
  (table[lhs_idx], table[rhs_idx])
}

fn hex_to_bytes(lhs: u8, rhs: u8) -> crate::Result<u8> {
  fn half(byte: u8) -> crate::Result<u8> {
    match byte {
      b'A'..=b'F' => Ok(byte.wrapping_sub(b'A').wrapping_add(10)),
      b'a'..=b'f' => Ok(byte.wrapping_sub(b'a').wrapping_add(10)),
      b'0'..=b'9' => Ok(byte.wrapping_sub(b'0')),
      _ => Err(HexError::InvalidHexCharacter.into()),
    }
  }
  Ok((half(lhs)? << 4) | half(rhs)?)
}

#[cfg(test)]
mod test {
  use crate::{
    collection::ArrayVectorU8,
    de::{HexDisplay, HexEncMode, decode_hex, encode_hex},
  };

  #[test]
  fn decode_has_correct_output() {
    assert_eq!(decode_hex(b"61626364", &mut [0; 4]).unwrap(), b"abcd");
    assert_eq!(decode_hex(b"0x6162636465", &mut [0; 5]).unwrap(), b"abcde");
    assert!(decode_hex(b"6", &mut [0, 0, 0, 0]).is_err());
  }

  #[test]
  fn encode_has_correct_output() {
    assert_eq!(encode_hex(&[], None, &mut [0u8; 8]).unwrap(), "0x");
    assert_eq!(encode_hex(b"AZ", None, &mut [0u8; 8]).unwrap(), "0x415a");
    assert_eq!(
      encode_hex(b"AZ", Some(HexEncMode::WithoutPrefixUpper), &mut [0u8; 8]).unwrap(),
      "415A"
    );
  }

  #[test]
  fn hex_display() {
    assert_eq!(
      &ArrayVectorU8::<u8, 16>::try_from(format_args!(
        "{}",
        HexDisplay(b"abcdZ", Some(HexEncMode::WithoutPrefixLower))
      ))
      .unwrap(),
      "616263645a".as_bytes()
    );
    assert_eq!(
      &ArrayVectorU8::<u8, 16>::try_from(format_args!(
        "{}",
        HexDisplay(b"abcdZ", Some(HexEncMode::WithPrefixLower))
      ))
      .unwrap(),
      "0x616263645a".as_bytes()
    );
  }
}
