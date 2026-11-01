//! Numeric string

use crate::{collections::ArrayStringU8, misc::AsciiGraphic};
use core::ops::{ControlFlow, DivAssign, Rem};

/// Array string that can store numbers composed by `8` bits.
pub type NumStr8 = ArrayStringU8<5>;
/// Array string that can store numbers composed by `16` bits.
pub type NumStr16 = ArrayStringU8<7>;
/// Array string that can store numbers composed by `32` bits.
pub type NumStr32 = ArrayStringU8<12>;
/// Array string that can store numbers composed by `64` bits.
pub type NumStr64 = ArrayStringU8<21>;

/// Formatting options for number-to-string conversion.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct NumStrFmt {
  /// Fill byte and its minimum width for padding.
  pub pad: Option<(AsciiGraphic, u8)>,
  /// Separator character and its digit position from the right.
  pub sep: Option<(AsciiGraphic, u8)>,
}

impl NumStrFmt {
  /// New empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { pad: None, sep: None }
  }

  /// Sets the fill byte and minimum width for padding.
  #[inline]
  #[must_use = must_use_itself_doc!()]
  pub const fn pad(mut self, fill: AsciiGraphic, width: u8) -> Self {
    self.pad = Some((fill, width));
    self
  }

  /// Sets the separator character and its digit position from the right.
  #[inline]
  #[must_use = must_use_itself_doc!()]
  pub const fn sep(mut self, char: AsciiGraphic, pos: u8) -> Self {
    self.sep = Some((char, pos));
    self
  }
}

/// Number as a string.
pub trait NumStr: Copy {
  /// Capacity.
  const CAP: usize;
  /// Whether the number type is signed.
  const IS_SIGNED: bool;
  /// Ten represented as the unsigned absolute-value type.
  const U_TEN: Self::UnsignedAbs;
  /// Zero represented as the unsigned absolute-value type.
  const U_ZERO: Self::UnsignedAbs;

  /// Unsigned absolute-value.
  type UnsignedAbs: Copy + DivAssign + PartialEq + Rem<Output = Self::UnsignedAbs> + TryInto<u8>;

  /// Whether the value is negative.
  fn is_neg(self) -> bool;

  /// Absolute value as an unsigned value.
  fn unsigned_abs(self) -> Self::UnsignedAbs;
}

#[expect(clippy::arithmetic_side_effects, reason = "% and / will never overflow with 10")]
#[inline(always)]
fn iteration<T, const CAP: usize>(
  buffer: &mut [u8; CAP],
  cap_u8: u8,
  idx: &mut u8,
  local_idx: u8,
  unsigned_abs: &mut T::UnsignedAbs,
) -> ControlFlow<()>
where
  T: NumStr,
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

fn num_string<const CAP: usize, T>(
  fill: AsciiGraphic,
  sep: Option<(AsciiGraphic, u8)>,
  value: T,
  width: u8,
) -> ArrayStringU8<CAP>
where
  T: NumStr,
{
  const {
    assert!(CAP >= T::CAP);
  }
  let cap_u8: u8 = CAP.try_into().unwrap_or(255);
  let is_neg = value.is_neg();
  let mut unsigned_abs = value.unsigned_abs();
  let mut buffer = [fill.as_u8(); CAP];
  let mut idx = cap_u8;
  let mut iter = 1..=cap_u8;
  if let Some((sep_ascii, sep_pos @ 1..=255)) = sep {
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
  // SAFETY: Numbers, ASCII fill, and AsciiGraphic separators are all valid ASCII (valid UTF-8).
  unsafe { ArrayStringU8::from_parts_unchecked(buffer, Some(len)) }
}

macro_rules! create_functions {
  ($(($fn_name:ident, $fn_name_fmt:ident, $ret_ty:ident)),* $(,)?) => {
    $(
      #[doc = concat!("Transforms a number into a [`", stringify!($ret_ty), "`].")]
      #[inline(always)]
      pub fn $fn_name<T>(value: T) -> $ret_ty
      where
        T: NumStr
      {
        num_string(AsciiGraphic::ZERO, None, value, 0)
      }

      #[doc = concat!("Transforms a number into a [`", stringify!($ret_ty), "`] using formatting options.")]
      #[inline(always)]
      pub fn $fn_name_fmt<T>(value: T, fmt: NumStrFmt) -> $ret_ty
      where
        T: NumStr
      {
        let (fill, width) = fmt.pad.unwrap_or((AsciiGraphic::ZERO, 0));
        num_string(fill, fmt.sep, value, width)
      }
    )*
  };
}

macro_rules! impl_num_str {
  ($(($sty:ty, $uty:ty)),* $(,)?) => {
    $(
      impl NumStr for $sty {
        const CAP: usize = <$uty>::CAP + 1;
        const IS_SIGNED: bool = true;
        const U_TEN: Self::UnsignedAbs = <$uty>::U_TEN;
        const U_ZERO: Self::UnsignedAbs = <$uty>::U_ZERO;
        type UnsignedAbs = $uty;

        #[inline] fn is_neg(self) -> bool { self < 0 }
        #[inline] fn unsigned_abs(self) -> Self::UnsignedAbs { <$sty>::unsigned_abs(self) }
      }

      impl NumStr for $uty {
        const CAP: usize = crate::misc::int_conv::u32usize(<$uty>::MAX.ilog10()) + 1;
        const IS_SIGNED: bool = false;
        const U_TEN: Self::UnsignedAbs = 10;
        const U_ZERO: Self::UnsignedAbs = 0;
        type UnsignedAbs = $uty;

        #[inline] fn is_neg(self) -> bool { false }
        #[inline] fn unsigned_abs(self) -> Self::UnsignedAbs { self }
      }
    )*
  };
}

create_functions!(
  (nstr8, nstr8_fmt, NumStr8),
  (nstr16, nstr16_fmt, NumStr16),
  (nstr32, nstr32_fmt, NumStr32),
  (nstr64, nstr64_fmt, NumStr64),
);

impl_num_str!((i8, u8), (i16, u16), (i32, u32), (i64, u64));

#[cfg(test)]
mod tests {
  use crate::{
    codec::num_str::{NumStrFmt, nstr8, nstr16, nstr16_fmt, nstr32, nstr32_fmt, nstr64},
    misc::AsciiGraphic,
  };

  #[test]
  fn edge_cases() {
    assert_eq!(nstr32_fmt(5i32, NumStrFmt::new().sep(AsciiGraphic::DOT, 2)), "0.05");
    assert_eq!(nstr32_fmt(-5i32, NumStrFmt::new().sep(AsciiGraphic::DOT, 2)), "-0.05");
    assert_eq!(nstr32_fmt(0i32, NumStrFmt::new().sep(AsciiGraphic::DOT, 2)), "0.00");
    assert_eq!(
      nstr32_fmt(-5i32, NumStrFmt::new().pad(AsciiGraphic::ZERO, 6).sep(AsciiGraphic::DOT, 2)),
      "-000.05"
    );
  }

  #[test]
  fn pad() {
    assert_eq!(nstr16_fmt(5i16, NumStrFmt::new().pad(AsciiGraphic::ZERO, 4)), "0005");
    assert_eq!(nstr16_fmt(-5i16, NumStrFmt::new().pad(AsciiGraphic::ZERO, 4)), "-0005");
    assert_eq!(nstr16_fmt(0i16, NumStrFmt::new().pad(AsciiGraphic::ZERO, 3)), "000");
    assert_eq!(nstr16_fmt(1234i16, NumStrFmt::new().pad(AsciiGraphic::ZERO, 2)), "1234");
    assert_eq!(nstr16_fmt(-1234i16, NumStrFmt::new().pad(AsciiGraphic::ZERO, 5)), "-01234");
  }

  #[test]
  fn sep() {
    assert_eq!(nstr32_fmt(1234567i32, NumStrFmt::new().sep(AsciiGraphic::DOT, 2)), "12345.67");
  }

  #[test]
  fn signed() {
    assert_eq!(nstr8(127i8), "127");
    assert_eq!(nstr8(12i8), "12");
    assert_eq!(nstr8(-0i8), "0");
    assert_eq!(nstr8(-12i8), "-12");
    assert_eq!(nstr8(-128i8), "-128");

    assert_eq!(nstr16(32767i16), "32767");
    assert_eq!(nstr16(3276i16), "3276");
    assert_eq!(nstr16(12i16), "12");
    assert_eq!(nstr16(-0i16), "0");
    assert_eq!(nstr16(-12i16), "-12");
    assert_eq!(nstr16(-3276i16), "-3276");
    assert_eq!(nstr16(-32768i16), "-32768");

    assert_eq!(nstr32(2147483647i32), "2147483647");
    assert_eq!(nstr32(214748364i32), "214748364");
    assert_eq!(nstr32(12i32), "12");
    assert_eq!(nstr32(-0i32), "0");
    assert_eq!(nstr32(-12i32), "-12");
    assert_eq!(nstr32(-214748364i32), "-214748364");
    assert_eq!(nstr32(-2147483648i32), "-2147483648");

    assert_eq!(nstr64(9223372036854775807i64), "9223372036854775807");
    assert_eq!(nstr64(922337203685477580i64), "922337203685477580");
    assert_eq!(nstr64(12i64), "12");
    assert_eq!(nstr64(-0i64), "0");
    assert_eq!(nstr64(-12i64), "-12");
    assert_eq!(nstr64(-922337203685477580i64), "-922337203685477580");
    assert_eq!(nstr64(-9223372036854775808i64), "-9223372036854775808");
  }

  #[test]
  fn unsigned() {
    assert_eq!(nstr8(0u8), "0");
    assert_eq!(nstr8(12u8), "12");
    assert_eq!(nstr8(255u8), "255");

    assert_eq!(nstr16(0u16), "0");
    assert_eq!(nstr16(12u16), "12");
    assert_eq!(nstr16(6553u16), "6553");
    assert_eq!(nstr16(65535u16), "65535");

    assert_eq!(nstr32(0u32), "0");
    assert_eq!(nstr32(12u32), "12");
    assert_eq!(nstr32(429496729u32), "429496729");
    assert_eq!(nstr32(4294967295u32), "4294967295");

    assert_eq!(nstr64(0u64), "0");
    assert_eq!(nstr64(12u64), "12");
    assert_eq!(nstr64(1844674407370955161u64), "1844674407370955161");
    assert_eq!(nstr64(18446744073709551615u64), "18446744073709551615");
  }
}
