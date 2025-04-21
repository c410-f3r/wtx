use crate::misc::ArrayString;
use core::ops::{DivAssign, Rem};

pub(crate) type I16String = ArrayString<5>;
pub(crate) type U16String = ArrayString<5>;
pub(crate) type U32String = ArrayString<10>;
pub(crate) type U64String = ArrayString<20>;

/// Transforms an `i16` into an [`ArrayString`].
#[inline]
pub fn i16_string(value: i16) -> I16String {
  num_string::<5, 5, i16>(value)
}

/// Transforms an `u16` into an [`ArrayString`].
#[inline]
pub fn u16_string(value: u16) -> U16String {
  num_string::<5, 5, u16>(value)
}

/// Transforms an `u32` into an [`ArrayString`].
#[inline]
pub fn u32_string(value: u32) -> U32String {
  num_string::<10, 10, u32>(value)
}

/// Fills an `u64` into an [`ArrayString`].
#[inline]
pub fn u64_string(value: u64) -> U64String {
  num_string::<20, 20, u64>(value)
}

#[expect(
  clippy::arithmetic_side_effects,
  reason = "% and / will never overflow with 5, 10 and 20 integer literals"
)]
#[inline]
fn num_string<const U8: u8, const USIZE: usize, T>(mut value: T) -> ArrayString<USIZE>
where
  T: Copy + DivAssign + From<u8> + PartialEq + Rem<Output = T>,
  u8: TryFrom<T>,
{
  let ten = T::from(10);
  let zero = T::from(0);
  let mut idx: u8 = U8;
  let mut buffer = [0; USIZE];
  for local_idx in 1..=U8 {
    idx = U8.wrapping_sub(local_idx);
    let Some(elem) = buffer.get_mut(usize::from(idx)) else {
      break;
    };
    *elem = u8::try_from(value % ten).unwrap_or_default().wrapping_add(48);
    value /= ten;
    if value == zero {
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
  use crate::misc::u64_string;

  #[test]
  fn num_array() {
    assert_eq!(u64_string(0).as_str(), "0");
    assert_eq!(u64_string(12).as_str(), "12");
    assert_eq!(u64_string(1844674407370955161).as_str(), "1844674407370955161");
    assert_eq!(u64_string(18446744073709551615).as_str(), "18446744073709551615");
  }
}
