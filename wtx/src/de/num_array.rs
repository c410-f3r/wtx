use crate::collection::ArrayString;
use core::ops::{DivAssign, Rem};

/// Array string that can store an `i16` number.
pub type I16String = ArrayString<6>;
/// Array string that can store an `u8` number.
pub type U8String = ArrayString<3>;
/// Array string that can store an `u16` number.
pub type U16String = ArrayString<5>;
/// Array string that can store an `u32` number.
pub type U32String = ArrayString<10>;
/// Array string that can store an `u64` number.
pub type U64String = ArrayString<20>;

/// Transforms an `i16` into an [`ArrayString`].
#[inline]
pub fn i16_string(value: i16) -> I16String {
  num_string::<true, 6, 6, i16>(value, i16::abs)
}

/// Transforms an `u8` into an [`ArrayString`].
#[inline]
pub fn u8_string(value: u8) -> U8String {
  num_string::<false, 3, 3, u8>(value, |el| el)
}

/// Transforms an `u16` into an [`ArrayString`].
#[inline]
pub fn u16_string(value: u16) -> U16String {
  num_string::<false, 5, 5, u16>(value, |el| el)
}

/// Transforms an `u32` into an [`ArrayString`].
#[inline]
pub fn u32_string(value: u32) -> U32String {
  num_string::<false, 10, 10, u32>(value, |el| el)
}

/// Fills an `u64` into an [`ArrayString`].
#[inline]
pub fn u64_string(value: u64) -> U64String {
  num_string::<false, 20, 20, u64>(value, |el| el)
}

#[expect(
  clippy::arithmetic_side_effects,
  reason = "% and / will never overflow with 5, 10 and 20 integer literals"
)]
fn num_string<const IS_SIGNED: bool, const U8: u8, const USIZE: usize, T>(
  mut value: T,
  mut abs: impl FnMut(T) -> T,
) -> ArrayString<USIZE>
where
  T: Copy + DivAssign + From<u8> + PartialEq + PartialOrd + Rem<Output = T>,
  u8: TryFrom<T>,
{
  let zero = T::from(0);
  if value == zero {
    // SAFETY: '0' is ASCII
    return unsafe { ArrayString::from_parts_unchecked([b'0'; USIZE], 1) };
  }
  let ten = T::from(10);
  let is_neg = value < zero;
  if IS_SIGNED {
    value = abs(value);
  }
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
  // SAFETY: Numbers are ASCII
  unsafe { ArrayString::from_parts_unchecked(data, len.into()) }
}

#[cfg(test)]
pub(crate) mod tests {
  use crate::de::{i16_string, u64_string};

  #[test]
  fn num_array() {
    assert_eq!(u64_string(0).as_str(), "0");
    assert_eq!(u64_string(12).as_str(), "12");
    assert_eq!(u64_string(1844674407370955161).as_str(), "1844674407370955161");
    assert_eq!(u64_string(18446744073709551615).as_str(), "18446744073709551615");
  }

  #[test]
  fn num_array_negative() {
    assert_eq!(i16_string(-0).as_str(), "0");
    assert_eq!(i16_string(-12).as_str(), "-12");
    assert_eq!(i16_string(-3276).as_str(), "-3276");
    assert_eq!(i16_string(-32767).as_str(), "-32767");
  }
}
