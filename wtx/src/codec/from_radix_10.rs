// Several experiments showed that carrying an error flag and only checking it at the end of an
// iteration actually slows down benchmarks. For some reason short-circuiting (branching) inside
// loops generate fewer and faster assembly.

#[cfg(all(feature = "_bench", test))]
mod bench;

use crate::misc::int_conv::u32usize;
use core::hint::cold_path;

/// Errors of [`FromRadix10`] implementations.
#[derive(Clone, Copy, Debug)]
pub enum FromRadix10Error {
  /// One of the bytes is not a number.
  ByteNaN,
  /// Passed set of bytes is empty.
  EmptyBytes,
  /// The underlying bytes represent a number greater than its associated type capacity.
  Overflow,
  /// Unsigned number can not have initial negative signs
  UnexpectedSign,
  /// Passed set of bytes has a length greater than the maximum available number of digits.
  VeryLargeBytesLen,
}

/// Tries to convert a set of bytes into an integer value.
pub trait FromRadix10: Sized {
  /// Tries to convert a set of bytes into an integer value.
  fn from_radix_10(bytes: &[u8]) -> crate::Result<Self>;
}

#[inline(always)]
fn ascii_to_number(byte: u8) -> Option<u8> {
  let digit = byte.wrapping_sub(b'0');
  if digit > 9 {
    cold_path();
    None
  } else {
    Some(digit)
  }
}

#[inline(always)]
fn parse_bytes<T>(bytes: &[u8]) -> crate::Result<T>
where
  T: LocalArithmetic,
{
  macro_rules! fast_path {
    ($digits:expr, $method:ident, $result:ident) => {
      for byte in $digits {
        let Some(digit) = ascii_to_number(*byte) else {
          cold_path();
          return Err(FromRadix10Error::ByteNaN.into());
        };
        $result = $result.wrapping_mul(10).$method(digit);
      }
    };
  }
  macro_rules! slow_path {
    ($last_byte:ident, $method:ident, $result:ident) => {
      let Some(last_digit) = ascii_to_number(*$last_byte) else {
        cold_path();
        return Err(FromRadix10Error::ByteNaN.into());
      };
      let (mul_result, has_mul_err) = $result.overflowing_mul(10);
      let (final_result, has_final_err) = mul_result.$method(last_digit);
      if has_mul_err || has_final_err {
        cold_path();
        return Err(FromRadix10Error::Overflow.into());
      }
      $result = final_result;
    };
  }

  let (is_positive, digits) = match bytes.split_first() {
    Some((b'+', digits)) => (true, digits),
    Some((b'-', digits)) => {
      if !T::IS_SIGNED {
        cold_path();
        return Err(FromRadix10Error::UnexpectedSign.into());
      }
      (false, digits)
    }
    Some(_) => (true, bytes),
    None => {
      cold_path();
      return Err(FromRadix10Error::EmptyBytes.into());
    }
  };
  let len = digits.len();
  let ([prefix @ .., last_byte], true) = (digits, len <= T::MAX_SAFE_LEN.wrapping_add(1)) else {
    cold_path();
    return Err(if len == 0 {
      FromRadix10Error::EmptyBytes.into()
    } else {
      FromRadix10Error::VeryLargeBytesLen.into()
    });
  };
  let mut result = T::ZERO;
  if is_positive {
    if len <= T::MAX_SAFE_LEN {
      fast_path!(digits, wrapping_add, result);
    } else {
      fast_path!(prefix, wrapping_add, result);
      slow_path!(last_byte, overflowing_add, result);
    }
  } else {
    if len <= T::MAX_SAFE_LEN {
      fast_path!(digits, wrapping_sub, result);
    } else {
      fast_path!(prefix, wrapping_sub, result);
      slow_path!(last_byte, overflowing_sub, result);
    }
  }
  Ok(result)
}

trait LocalArithmetic: Copy + Sized {
  const IS_SIGNED: bool;
  const MAX_SAFE_LEN: usize;
  const ZERO: Self;
  fn overflowing_add(self, other: u8) -> (Self, bool);
  fn overflowing_mul(self, other: u8) -> (Self, bool);
  fn overflowing_sub(self, other: u8) -> (Self, bool);
  fn wrapping_add(self, other: u8) -> Self;
  fn wrapping_mul(self, other: u8) -> Self;
  fn wrapping_sub(self, other: u8) -> Self;
}

macro_rules! implement {
  ($macro_counter:ident, $($ty:ty),+) => {
    $(
      #[allow(
        clippy::as_conversions,
        clippy::cast_possible_wrap,
        trivial_numeric_casts,
        unused_comparisons,
        reason = "depends on the input"
      )]
      impl LocalArithmetic for $ty {
        const IS_SIGNED: bool = 0 > <$ty>::MIN;
        const MAX_SAFE_LEN: usize = u32usize(<$ty>::MAX.ilog10());
        const ZERO: Self = 0;
        #[inline(always)]
        fn overflowing_add(self, other: u8) -> (Self, bool) { self.overflowing_add(other as $ty) }
        #[inline(always)]
        fn overflowing_mul(self, other: u8) -> (Self, bool) { self.overflowing_mul(other as $ty) }
        #[inline(always)]
        fn overflowing_sub(self, other: u8) -> (Self, bool) { self.overflowing_sub(other as $ty) }
        #[inline(always)]
        fn wrapping_add(self, other: u8) -> Self { self.wrapping_add(other as $ty) }
        #[inline(always)]
        fn wrapping_mul(self, other: u8) -> Self { self.wrapping_mul(other as $ty) }
        #[inline(always)]
        fn wrapping_sub(self, other: u8) -> Self { self.wrapping_sub(other as $ty) }
      }

      impl FromRadix10 for $ty {
        #[inline(always)]
        fn from_radix_10(bytes: &[u8]) -> crate::Result<Self> {
          parse_bytes(bytes)
        }
      }
    )+
  };
}

implement!(count_max_negative_digits, i8, i16, i32, i64, i128, isize);
implement!(count_max_positive_digits, u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod test {
  use crate::codec::FromRadix10;

  #[test]
  fn has_correct_outputs() {
    assert_eq!(u8::from_radix_10(b"0").unwrap(), 0);
    assert_eq!(u8::from_radix_10(b"000").unwrap(), 0);
    assert_eq!(u8::from_radix_10(b"25").unwrap(), 25);
    assert_eq!(u8::from_radix_10(b"255").unwrap(), 255);
    assert!(u8::from_radix_10(b"").is_err());
    assert!(u8::from_radix_10(b"-").is_err());
    assert!(u8::from_radix_10(b"-0").is_err());
    assert!(u8::from_radix_10(b"25foo").is_err());
    assert!(u8::from_radix_10(b"1000").is_err());

    assert_eq!(i8::from_radix_10(b"0").unwrap(), 0);
    assert_eq!(i8::from_radix_10(b"000").unwrap(), 0);
    assert_eq!(i8::from_radix_10(b"-0").unwrap(), 0);
    assert_eq!(i8::from_radix_10(b"25").unwrap(), 25);
    assert_eq!(i8::from_radix_10(b"-25").unwrap(), -25);
    assert_eq!(i8::from_radix_10(b"127").unwrap(), 127);
    assert_eq!(i8::from_radix_10(b"-127").unwrap(), -127);
    assert_eq!(i8::from_radix_10(b"-128").unwrap(), -128);
    assert!(i8::from_radix_10(b"").is_err());
    assert!(i8::from_radix_10(b"-").is_err());
    assert!(i8::from_radix_10(b"25foo").is_err());
    assert!(i8::from_radix_10(b"-25foo").is_err());
    assert!(i8::from_radix_10(b"128").is_err());
    assert!(i8::from_radix_10(b"-129").is_err());
    assert!(i8::from_radix_10(b"1000").is_err());
    assert!(i8::from_radix_10(b"-1000").is_err());
  }
}
