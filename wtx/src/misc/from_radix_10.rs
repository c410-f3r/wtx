macro_rules! ascii_to_number {
  ($ch:expr) => {
    match $ch {
      b'0' => Ok(0),
      b'1' => Ok(1),
      b'2' => Ok(2),
      b'3' => Ok(3),
      b'4' => Ok(4),
      b'5' => Ok(5),
      b'6' => Ok(6),
      b'7' => Ok(7),
      b'8' => Ok(8),
      b'9' => Ok(9),
      _ => Err(FromRadix10Error::ByteNaN),
    }
  };
}

macro_rules! count_max_negative_digits {
  ($ty:ty) => {{
    let mut digits: u8 = 0;
    let mut value = <$ty>::MIN;
    while value < 0 {
      digits = digits.wrapping_add(1);
      value /= 10;
    }
    digits
  }};
}

macro_rules! count_max_positive_digits {
  ($ty:ty) => {{
    let mut digits: u8 = 0;
    let mut value = <$ty>::MAX;
    while value > 0 {
      digits = digits.wrapping_add(1);
      value /= 10;
    }
    digits
  }};
}

macro_rules! impl_signed {
  ($($ty:ty)+) => {
    $(
      impl FromRadix10 for $ty {
        #[inline]
        fn from_radix_10(bytes: &[u8]) -> crate::Result<Self> {
          signed(
            bytes,
            (count_max_negative_digits!($ty), count_max_positive_digits!($ty)),
            (0, 10),
            |byte| ascii_to_number!(byte)
          )
        }
      }
    )+
  };
}

macro_rules! impl_unsigned {
  ($($ty:ty)+) => {
    $(
      impl FromRadix10 for $ty {
        #[inline]
        fn from_radix_10(bytes: &[u8]) -> crate::Result<Self> {
          unsigned(bytes, count_max_positive_digits!($ty), (0, 10), |byte| ascii_to_number!(byte))
        }
      }
    )+
  };
}

use core::{
  num::Wrapping,
  ops::{Add, Mul, Neg},
};

/// Errors of [`FromRadix10`] implementations.
#[derive(Debug)]
pub enum FromRadix10Error {
  /// One of the bytes is not a number.
  ByteNaN,
  /// Passed set of bytes is empty.
  EmptyBytes,
  /// Passed set of bytes has a length greater than the maximum available number of digits.
  VeryLargeBytesLen,
}

/// Tries to convert a set of bytes into an integer value.
pub trait FromRadix10: Sized {
  /// Tries to convert a set of bytes into an integer value.
  fn from_radix_10(bytes: &[u8]) -> crate::Result<Self>;
}

#[inline]
fn signed<T>(
  bytes: &[u8],
  (max_negative_digits, max_positive_digits): (u8, u8),
  (zero, ten): (T, T),
  mut cb: impl FnMut(u8) -> Result<T, FromRadix10Error>,
) -> crate::Result<T>
where
  T: Copy + Neg<Output = T>,
  Wrapping<T>: Add<Wrapping<T>, Output = Wrapping<T>>,
  Wrapping<T>: Mul<Wrapping<T>, Output = Wrapping<T>>,
{
  let mut iter = bytes.iter().copied();
  let Some(first_byte) = iter.next() else {
    return Err(FromRadix10Error::EmptyBytes.into());
  };
  if first_byte == b'-' {
    if iter.len() > max_negative_digits.into() {
      return Err(FromRadix10Error::VeryLargeBytesLen.into());
    }
    let mut rslt = Wrapping(zero);
    for byte in iter.take(max_negative_digits.into()) {
      let digit = cb(byte)?;
      rslt = rslt.mul(Wrapping(ten));
      rslt = rslt.add(Wrapping(digit));
    }
    #[expect(clippy::arithmetic_side_effects, reason = "false-positive")]
    Ok(-rslt.0)
  } else {
    if bytes.len() > max_positive_digits.into() {
      return Err(FromRadix10Error::VeryLargeBytesLen.into());
    }
    let mut digit = cb(first_byte)?;
    let mut rslt = Wrapping(digit);
    for byte in iter.take(max_positive_digits.into()) {
      digit = cb(byte)?;
      rslt = rslt.mul(Wrapping(ten));
      rslt = rslt.add(Wrapping(digit));
    }
    Ok(rslt.0)
  }
}

#[inline]
fn unsigned<T>(
  bytes: &[u8],
  max_positive_digits: u8,
  (zero, ten): (T, T),
  mut cb: impl FnMut(u8) -> Result<T, FromRadix10Error>,
) -> crate::Result<T>
where
  T: Copy,
  Wrapping<T>: Add<Wrapping<T>, Output = Wrapping<T>>,
  Wrapping<T>: Mul<Wrapping<T>, Output = Wrapping<T>>,
{
  if bytes.is_empty() {
    return Err(FromRadix10Error::EmptyBytes.into());
  }
  if bytes.len() > max_positive_digits.into() {
    return Err(FromRadix10Error::VeryLargeBytesLen.into());
  }
  let mut rslt = Wrapping(zero);
  for byte in bytes.iter().copied() {
    let digit = cb(byte)?;
    rslt = rslt.mul(Wrapping(ten));
    rslt = rslt.add(Wrapping(digit));
  }
  Ok(rslt.0)
}

impl_signed!(i8 i16 i32 i64 i128 isize);
impl_unsigned!(u8 u16 u32 u64 u128 usize);

#[cfg(test)]
mod test {
  use crate::misc::FromRadix10;

  #[test]
  fn has_correct_outputs() {
    assert_eq!(u8::from_radix_10(b"0").unwrap(), 0);
    assert_eq!(u8::from_radix_10(b"25").unwrap(), 25);
    assert_eq!(u8::from_radix_10(b"255").unwrap(), 255);
    assert!(u8::from_radix_10(b"").is_err());
    assert!(u8::from_radix_10(b"-0").is_err());
    assert!(u8::from_radix_10(b"25foo").is_err());
    assert!(u8::from_radix_10(b"1000").is_err());

    assert_eq!(i8::from_radix_10(b"0").unwrap(), 0);
    assert_eq!(i8::from_radix_10(b"-0").unwrap(), 0);
    assert_eq!(i8::from_radix_10(b"25").unwrap(), 25);
    assert_eq!(i8::from_radix_10(b"-25").unwrap(), -25);
    assert_eq!(i8::from_radix_10(b"127").unwrap(), 127);
    assert_eq!(i8::from_radix_10(b"-127").unwrap(), -127);
    assert!(i8::from_radix_10(b"").is_err());
    assert!(i8::from_radix_10(b"25foo").is_err());
    assert!(i8::from_radix_10(b"-25foo").is_err());
    assert!(i8::from_radix_10(b"1000").is_err());
    assert!(i8::from_radix_10(b"-1000").is_err());
  }
}
