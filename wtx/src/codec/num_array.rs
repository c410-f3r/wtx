use crate::{collections::ArrayStringU8, misc::AsciiGraphic};
use core::ops::{ControlFlow, DivAssign, Rem};

/// Array string that can store an `i8` number.
pub type I8String = ArrayStringU8<5>;
/// Array string that can store an `i16` number.
pub type I16String = ArrayStringU8<7>;
/// Array string that can store an `i32` number.
pub type I32String = ArrayStringU8<12>;
/// Array string that can store an `i64` number.
pub type I64String = ArrayStringU8<21>;

/// Array string that can store an `u8` number.
pub type U8String = ArrayStringU8<4>;
/// Array string that can store an `u16` number.
pub type U16String = ArrayStringU8<6>;
/// Array string that can store an `u32` number.
pub type U32String = ArrayStringU8<11>;
/// Array string that can store an `u64` number.
pub type U64String = ArrayStringU8<21>;

/// Transforms an `i8` into an [`ArrayStringU8`].
#[inline]
pub fn i8_string(value: i8) -> I8String {
  num_string(0, None, value, 0)
}
/// Transforms an `i16` into an [`ArrayStringU8`].
#[inline]
pub fn i16_string(value: i16) -> I16String {
  num_string(0, None, value, 0)
}
/// Transforms an `i32` into an [`ArrayStringU8`].
#[inline]
pub fn i32_string(value: i32) -> I32String {
  num_string(0, None, value, 0)
}
/// Transforms an `i64` into an [`ArrayStringU8`].
#[inline]
pub fn i64_string(value: i64) -> I64String {
  num_string(0, None, value, 0)
}

/// Transforms an `i16` into an [`ArrayStringU8`] with padding.
#[inline]
pub fn i16_string_pad(value: i16, fill: AsciiGraphic, width: u8) -> I16String {
  num_string(fill.into(), None, value, width)
}

/// Transforms an `u8` into an [`ArrayStringU8`].
#[inline]
pub fn u8_string(value: u8) -> U8String {
  num_string(0, None, value, 0)
}
/// Transforms an `u16` into an [`ArrayStringU8`].
#[inline]
pub fn u16_string(value: u16) -> U16String {
  num_string(0, None, value, 0)
}
/// Transforms an `u32` into an [`ArrayStringU8`].
#[inline]
pub fn u32_string(value: u32) -> U32String {
  num_string(0, None, value, 0)
}
/// Transforms an `u64` into an [`ArrayStringU8`].
#[inline]
pub fn u64_string(value: u64) -> U64String {
  num_string(0, None, value, 0)
}

/// Transforms an `u32` into an [`ArrayStringU8`] with padding.
#[inline]
pub fn u32_string_pad(value: u32, fill: AsciiGraphic, width: u8) -> U32String {
  num_string(fill.into(), None, value, width)
}

#[inline]
pub(crate) fn num_string<const CAP: usize, T>(
  fill: u8,
  sep: Option<(AsciiGraphic, u8)>,
  value: T,
  width: u8,
) -> ArrayStringU8<CAP>
where
  T: NumArray,
{
  const {
    assert!(CAP >= T::CAP);
  }
  let cap_u8: u8 = CAP.try_into().unwrap_or(255);
  let is_neg = value.is_neg();
  let mut unsigned_abs = value.unsigned_abs();
  let mut buffer = [fill; CAP];
  let mut idx = cap_u8;
  let mut iter = 1..=cap_u8;
  if let Some((sep_ascii, sep_pos)) = sep {
    for local_idx in iter.by_ref().take(usize::from(sep_pos)) {
      let _cf = iteration::<T, _>(&mut buffer, cap_u8, &mut idx, local_idx, &mut unsigned_abs);
    }
    if let Some(local_idx) = iter.next() {
      idx = cap_u8.wrapping_sub(local_idx);
      if let Some(num) = buffer.get_mut(usize::from(idx)) {
        *num = sep_ascii.into();
      }
    }
  }
  for local_idx in iter {
    if iteration::<T, _>(&mut buffer, cap_u8, &mut idx, local_idx, &mut unsigned_abs).is_break() {
      break;
    }
  }
  let mut len = cap_u8.wrapping_sub(idx);
  len = len.max(width);
  if T::IS_SIGNED && is_neg {
    len = len.saturating_add(1);
  }
  len = len.min(cap_u8);
  idx = cap_u8.wrapping_sub(len);
  if T::IS_SIGNED
    && is_neg
    && let Some(sign) = buffer.get_mut(usize::from(idx))
  {
    *sign = b'-';
  }
  buffer.copy_within(usize::from(idx).., 0);
  // SAFETY: Numbers are ASCII
  let mut rslt = unsafe { ArrayStringU8::from_parts_unchecked(buffer, Some(len)) };
  if rslt.as_bytes().last().copied() == sep.map(|el| el.0.into()) {
    let _ = rslt.pop();
  }
  rslt
}

#[expect(clippy::arithmetic_side_effects, reason = "% and / will never overflow with 10")]
fn iteration<T, const CAP: usize>(
  buffer: &mut [u8; CAP],
  cap_u8: u8,
  idx: &mut u8,
  local_idx: u8,
  unsigned_abs: &mut T::UnsignedAbs,
) -> ControlFlow<()>
where
  T: NumArray,
{
  *idx = cap_u8.wrapping_sub(local_idx);
  let Some(num) = buffer.get_mut(usize::from(*idx)) else {
    return ControlFlow::Break(());
  };
  let rem = *unsigned_abs % T::U_TEN;
  *num = rem.try_into().unwrap_or_default().wrapping_add(b'0');
  *unsigned_abs /= T::U_TEN;
  if *unsigned_abs == T::U_ZERO {
    return ControlFlow::Break(());
  }
  ControlFlow::Continue(())
}

pub(crate) trait NumArray: Copy {
  const CAP: usize;
  const IS_SIGNED: bool;
  const U_TEN: Self::UnsignedAbs;
  const U_ZERO: Self::UnsignedAbs;

  type UnsignedAbs: Copy + DivAssign + PartialEq + Rem<Output = Self::UnsignedAbs> + TryInto<u8>;

  fn is_neg(self) -> bool;

  fn unsigned_abs(self) -> Self::UnsignedAbs;
}

macro_rules! impl_num_array {
  (
    $(($sty:ty, $uty:ty)),*
  ) => {
    $(
      impl NumArray for $sty {
        const CAP: usize = <$uty>::CAP + 1;
        const IS_SIGNED: bool = true;
        const U_TEN: Self::UnsignedAbs = <$uty>::U_TEN;
        const U_ZERO: Self::UnsignedAbs = <$uty>::U_ZERO;

        type UnsignedAbs = $uty;

        #[inline]
        fn is_neg(self) -> bool {
          self < 0
        }

        #[inline]
        fn unsigned_abs(self) -> Self::UnsignedAbs {
          <$sty>::unsigned_abs(self)
        }
      }

      impl NumArray for $uty {
        const CAP: usize = crate::misc::int_conv::u32usize(<$uty>::MAX.ilog10()) + 1;
        const IS_SIGNED: bool = false;
        const U_TEN: Self::UnsignedAbs = 10;
        const U_ZERO: Self::UnsignedAbs = 0;

        type UnsignedAbs = $uty;

        #[inline]
        fn is_neg(self) -> bool {
          false
        }

        #[inline]
        fn unsigned_abs(self) -> Self::UnsignedAbs {
          self
        }
      }
    )*
  };
}

impl_num_array!((i8, u8), (i16, u16), (i32, u32), (i64, u64), (i128, u128));

#[cfg(test)]
pub(crate) mod tests {
  use crate::{
    codec::{
      i8_string, i16_string, i16_string_pad, i32_string, i64_string, num_string, u8_string,
      u16_string, u32_string, u64_string,
    },
    misc::AsciiGeneric,
  };

  #[test]
  fn pad() {
    assert_eq!(i16_string_pad(5, AsciiGeneric::ZERO.try_into().unwrap(), 4).as_str(), "0005");
    assert_eq!(i16_string_pad(-5, AsciiGeneric::ZERO.try_into().unwrap(), 4).as_str(), "-0005");
    assert_eq!(i16_string_pad(0, AsciiGeneric::ZERO.try_into().unwrap(), 3).as_str(), "000");
    assert_eq!(i16_string_pad(1234, AsciiGeneric::ZERO.try_into().unwrap(), 2).as_str(), "1234");
    assert_eq!(i16_string_pad(-1234, AsciiGeneric::ZERO.try_into().unwrap(), 5).as_str(), "-01234");
  }

  #[test]
  fn sep() {
    assert_eq!(
      num_string::<15, _>(0, Some((AsciiGeneric::DOT.try_into().unwrap(), 2)), 1234567, 0),
      "12345.67"
    );
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
}
