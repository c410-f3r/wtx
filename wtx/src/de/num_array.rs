use crate::collection::ArrayStringU8;
use core::ops::{DivAssign, Rem};

/// Array string that can store an `i8` number.
pub type I8String = ArrayStringU8<4>;
/// Array string that can store an `i16` number.
pub type I16String = ArrayStringU8<6>;
/// Array string that can store an `i32` number.
pub type I32String = ArrayStringU8<11>;
/// Array string that can store an `i64` number.
pub type I64String = ArrayStringU8<20>;

/// Array string that can store an `u8` number.
pub type U8String = ArrayStringU8<3>;
/// Array string that can store an `u16` number.
pub type U16String = ArrayStringU8<5>;
/// Array string that can store an `u32` number.
pub type U32String = ArrayStringU8<10>;
/// Array string that can store an `u64` number.
pub type U64String = ArrayStringU8<20>;

/// Transforms an `i8` into an [`ArrayStringU8`].
#[inline]
pub fn i8_string(value: i8) -> I8String {
  num_string::<true, 4, 4, u8>(value < 0, value.unsigned_abs())
}
/// Transforms an `i16` into an [`ArrayStringU8`].
#[inline]
pub fn i16_string(value: i16) -> I16String {
  num_string::<true, 6, 6, u16>(value < 0, value.unsigned_abs())
}
/// Transforms an `i32` into an [`ArrayStringU8`].
#[inline]
pub fn i32_string(value: i32) -> I32String {
  num_string::<true, 11, 11, u32>(value < 0, value.unsigned_abs())
}
/// Transforms an `i64` into an [`ArrayStringU8`].
#[inline]
pub fn i64_string(value: i64) -> I64String {
  num_string::<true, 20, 20, u64>(value < 0, value.unsigned_abs())
}

/// Transforms an `u8` into an [`ArrayStringU8`].
#[inline]
pub fn u8_string(value: u8) -> U8String {
  num_string::<false, 3, 3, u8>(false, value)
}
/// Transforms an `u16` into an [`ArrayStringU8`].
#[inline]
pub fn u16_string(value: u16) -> U16String {
  num_string::<false, 5, 5, u16>(false, value)
}
/// Transforms an `u32` into an [`ArrayStringU8`].
#[inline]
pub fn u32_string(value: u32) -> U32String {
  num_string::<false, 10, 10, u32>(false, value)
}
/// Fills an `u64` into an [`ArrayStringU8`].
#[inline]
pub fn u64_string(value: u64) -> U64String {
  num_string::<false, 20, 20, u64>(false, value)
}

#[expect(
  clippy::arithmetic_side_effects,
  reason = "% and / will never overflow with 5, 10 and 20 integer literals"
)]
#[inline]
fn num_string<const IS_SIGNED: bool, const U8: u8, const USIZE: usize, T>(
  is_neg: bool,
  mut value: T,
) -> ArrayStringU8<USIZE>
where
  T: Copy + DivAssign + From<u8> + PartialEq + PartialOrd + Rem<Output = T>,
  u8: TryFrom<T>,
{
  let zero = T::from(0);
  if value == zero {
    // SAFETY: '0' is ASCII
    return unsafe { ArrayStringU8::from_parts_unchecked([b'0'; USIZE], 1) };
  }
  let ten = T::from(10);
  let mut buffer = [0; USIZE];
  let mut idx: u8 = U8;
  for local_idx in 1..=U8 {
    idx = U8.wrapping_sub(local_idx);
    let Some(num) = buffer.get_mut(usize::from(idx)) else {
      break;
    };
    let rem = value % ten;
    *num = u8::try_from(rem).unwrap_or_default().wrapping_add(48);
    value /= ten;
    if value == zero {
      if IS_SIGNED && is_neg {
        idx = U8.wrapping_sub(local_idx.wrapping_add(1));
        if let Some(sign) = buffer.get_mut(usize::from(idx)) {
          *sign = b'-';
        }
      }
      break;
    }
  }
  let mut data = [0; USIZE];
  let len = U8.wrapping_sub(idx);
  let slice = data.get_mut(..usize::from(len)).unwrap_or_default();
  slice.copy_from_slice(buffer.get(usize::from(idx)..).unwrap_or_default());
  // SAFETY: numbers are ASCII
  unsafe { ArrayStringU8::from_parts_unchecked(data, len) }
}

#[cfg(test)]
pub(crate) mod tests {
  use crate::de::{
    i8_string, i16_string, i32_string, i64_string, u8_string, u16_string, u32_string, u64_string,
  };

  #[test]
  fn unsigned() {
    assert_eq!(u8_string(0).as_str(), "0");
    assert_eq!(u8_string(12).as_str(), "12");
    assert_eq!(u8_string(255).as_str(), "255");

    assert_eq!(u16_string(0).as_str(), "0");
    assert_eq!(u16_string(12).as_str(), "12");
    assert_eq!(u16_string(6553).as_str(), "6553");
    assert_eq!(u16_string(65535).as_str(), "65535");

    assert_eq!(u32_string(0).as_str(), "0");
    assert_eq!(u32_string(12).as_str(), "12");
    assert_eq!(u32_string(429496729).as_str(), "429496729");
    assert_eq!(u32_string(4294967295).as_str(), "4294967295");

    assert_eq!(u64_string(0).as_str(), "0");
    assert_eq!(u64_string(12).as_str(), "12");
    assert_eq!(u64_string(1844674407370955161).as_str(), "1844674407370955161");
    assert_eq!(u64_string(18446744073709551615).as_str(), "18446744073709551615");
  }

  #[test]
  fn signed() {
    assert_eq!(i8_string(127).as_str(), "127");
    assert_eq!(i8_string(12).as_str(), "12");
    assert_eq!(i8_string(-0).as_str(), "0");
    assert_eq!(i8_string(-12).as_str(), "-12");
    assert_eq!(i8_string(-128).as_str(), "-128");

    assert_eq!(i16_string(32767).as_str(), "32767");
    assert_eq!(i16_string(3276).as_str(), "3276");
    assert_eq!(i16_string(12).as_str(), "12");
    assert_eq!(i16_string(-0).as_str(), "0");
    assert_eq!(i16_string(-12).as_str(), "-12");
    assert_eq!(i16_string(-3276).as_str(), "-3276");
    assert_eq!(i16_string(-32768).as_str(), "-32768");

    assert_eq!(i32_string(2147483647).as_str(), "2147483647");
    assert_eq!(i32_string(214748364).as_str(), "214748364");
    assert_eq!(i32_string(12).as_str(), "12");
    assert_eq!(i32_string(-0).as_str(), "0");
    assert_eq!(i32_string(-12).as_str(), "-12");
    assert_eq!(i32_string(-214748364).as_str(), "-214748364");
    assert_eq!(i32_string(-2147483648).as_str(), "-2147483648");

    assert_eq!(i64_string(9223372036854775807).as_str(), "9223372036854775807");
    assert_eq!(i64_string(922337203685477580).as_str(), "922337203685477580");
    assert_eq!(i64_string(12).as_str(), "12");
    assert_eq!(i64_string(-0).as_str(), "0");
    assert_eq!(i64_string(-12).as_str(), "-12");
    assert_eq!(i64_string(-922337203685477580).as_str(), "-922337203685477580");
    assert_eq!(i64_string(-9223372036854775808).as_str(), "-9223372036854775808");
  }
}
