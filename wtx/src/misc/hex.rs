use crate::collection::{ArrayString, ArrayVector};
use core::fmt::{Display, Formatter};

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
pub struct HexDisplay<'bytes, const HAS_PREFIX: bool>(pub &'bytes [u8]);

impl<const HAS_PREFIX: bool> Display for HexDisplay<'_, HAS_PREFIX> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    if HAS_PREFIX {
      write!(f, "0x")?;
    }
    for &byte in self.0 {
      write!(f, "{byte:02x}")?;
    }
    Ok(())
  }
}

impl<'bytes, const HAS_PREFIX: bool, const N: usize> TryFrom<HexDisplay<'bytes, HAS_PREFIX>>
  for ArrayString<N>
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: HexDisplay<'bytes, HAS_PREFIX>) -> Result<Self, Self::Error> {
    ArrayString::try_from(format_args!("{from}"))
  }
}

impl<'bytes, const HAS_PREFIX: bool, const N: usize> TryFrom<HexDisplay<'bytes, HAS_PREFIX>>
  for ArrayVector<u8, N>
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: HexDisplay<'bytes, HAS_PREFIX>) -> Result<Self, Self::Error> {
    ArrayVector::try_from(format_args!("{from}"))
  }
}

/// Decodes `data` into `out` returning the affected part.
#[inline]
pub fn decode_hex_to_slice<'out, const HAS_PREFIX: bool>(
  mut data: &[u8],
  out: &'out mut [u8],
) -> crate::Result<&'out mut [u8]> {
  data = if HAS_PREFIX { data.get(2..).unwrap_or_default() } else { data };
  let (slice, rem) = data.as_chunks::<2>();
  if !rem.is_empty() {
    return Err(HexError::OddLen.into());
  }
  let idx = data.len() / 2;
  if idx > out.len() {
    return Err(HexError::InsufficientBuffer.into());
  }
  for ([a, b], byte) in slice.iter().zip(&mut *out) {
    let lhs = hex_byte(*a)? << 4;
    let rhs = hex_byte(*b)?;
    *byte = lhs | rhs;
  }
  Ok(out.get_mut(..idx).unwrap_or_default())
}

fn hex_byte(byte: u8) -> crate::Result<u8> {
  match byte {
    b'A'..=b'F' => Ok(byte.wrapping_sub(b'A').wrapping_add(10)),
    b'a'..=b'f' => Ok(byte.wrapping_sub(b'a').wrapping_add(10)),
    b'0'..=b'9' => Ok(byte.wrapping_sub(b'0')),
    _ => Err(HexError::InvalidHexCharacter.into()),
  }
}

#[cfg(test)]
mod test {
  use crate::{
    collection::ArrayVector,
    misc::{HexDisplay, decode_hex_to_slice},
  };

  #[test]
  fn decode_hex_to_slice_has_correct_output() {
    {
      let mut bufer = ArrayVector::from_array([0; 8]);
      let _ = decode_hex_to_slice::<false>(b"61626364", &mut bufer).unwrap();
      assert_eq!(bufer.as_slice(), b"abcd\0\0\0\0");
    }
    {
      let mut bufer = ArrayVector::from_array([0; 8]);
      let _ = decode_hex_to_slice::<true>(b"0x6162636465", &mut bufer).unwrap();
      assert_eq!(bufer.as_slice(), b"abcde\0\0\0");
    }
    {
      assert!(decode_hex_to_slice::<false>(b"6", &mut [0, 0, 0, 0]).is_err());
    }
  }

  #[test]
  fn hex_display() {
    assert_eq!(
      &ArrayVector::<u8, 16>::try_from(format_args!("{}", HexDisplay::<false>(b"abcd"))).unwrap(),
      "61626364".as_bytes()
    );
    assert_eq!(
      &ArrayVector::<u8, 16>::try_from(format_args!("{}", HexDisplay::<true>(b"abcd"))).unwrap(),
      "0x61626364".as_bytes()
    );
  }
}
