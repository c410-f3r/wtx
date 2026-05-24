use crate::{
  codec::i32_string,
  misc::int_conv::{i8i32, i16i32, i32f64, u8i32},
};
use core::fmt::{Debug, Display, Formatter, Write as _};
#[cfg(feature = "rust_decimal")]
use rust_decimal::Decimal;

/// Parts Per Million (PPM).
///
/// * 1₁₀  = 10²%    = 10⁴bps = 10⁶ppm
/// * 1ppm = 10⁻²bps = 10⁻⁴%  = 10⁻⁶₁₀
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ppm {
  value: i32,
}

impl Ppm {
  /// `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  pub const MAX: Self = Self { value: 2_147_483_647 };
  /// `-2_147.483647₁₀` | `-214_748.3647%` | `-21_474_836.47bps` | `-2_147_483_647ppm`
  pub const MIN: Self = Self { value: -2_147_483_647 };
  /// `1₁₀` | `100%` | `10_000bps` | `1_000_000ppm`
  pub const ONE_MILLION: Self = Self { value: 1_000_000 };
  /// `0₁₀` | `0%` | `0bps` | `0ppm`
  pub const ZERO: Self = Self { value: 0 };

  /// From `-3.2768₁₀` | `-324.68%` | `-32_768bps` | `-3_276_800ppm`
  /// To   `+3.2767₁₀` | `+324.67%` | `+32_767bps` | `+3_276_700ppm`
  pub const fn from_bps_i16(value: i16) -> Self {
    Self { value: i16i32(value).wrapping_mul(100) }
  }

  /// From `-2_147.483647₁₀` | `-214_748.3647%` | `-21_474_836.47bps` | `-2_147_483_647ppm`
  /// To   `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  #[cfg(feature = "rust_decimal")]
  pub fn from_decimal_decimal(from: Decimal) -> crate::Result<Self> {
    const ONE_MILLION: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);
    let fun = || Self::from_ppm_i32(i32::try_from(from.checked_mul(ONE_MILLION)?).ok()?).ok();
    fun().ok_or(crate::Error::InvalidPpmValue)
  }

  /// From `-128₁₀` | `-12_800%` | `-1_280_000bps` | `-128_000_000ppm`
  /// To   `+127₁₀` | `+12_700%` | `+1_270_000bps` | `+127_000_000ppm`
  pub const fn from_decimal_i8(from: i8) -> Self {
    Self { value: i8i32(from).wrapping_mul(1_000_000) }
  }

  /// From `  0₁₀` |      `0%` |         `0bps` |           `0ppm`
  /// To   `255₁₀` | `25_500%` | `2_550_000bps` | `255_000_000ppm`
  pub const fn from_decimal_u8(from: u8) -> Self {
    Self { value: u8i32(from).wrapping_mul(1_000_000) }
  }

  /// From `-327.68₁₀` | `-32_768%` | `-3_276_800bps` | `-327_680_000ppm`
  /// To   `+327.67₁₀` | `+32_767%` | `+3_276_700bps` | `+327_670_000ppm`
  pub const fn from_pct_i16(value: i16) -> Self {
    Self { value: i16i32(value).wrapping_mul(10_000) }
  }

  /// From `-0.032768₁₀` | `-3.2768%` | `-327.68bps` | `-32_768ppm`
  /// To   `+0.032767₁₀` | `+3.2767%` | `+327.67bps` | `+32_767ppm`
  pub const fn from_ppm_i16(value: i16) -> Self {
    Self { value: i16i32(value) }
  }

  /// From `-2_147.483647₁₀` | `-214_748.3647%` | `-21_474_836.47bps` | `-2_147_483_647ppm`
  /// To   `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  pub fn from_ppm_i32(value: i32) -> crate::Result<Self> {
    if !(Self::MIN.value..=Self::MAX.value).contains(&value) {
      return Err(crate::Error::InvalidPpmValue);
    }
    Ok(Self { value })
  }

  /// For example, if Ppm is 2%, then its complement value is 98%.
  #[cfg(feature = "rust_decimal")]
  pub fn complement_decimal_decimal(self) -> Decimal {
    Decimal::ONE.saturating_sub(self.decimal_decimal())
  }

  #[cfg(feature = "rust_decimal")]
  pub const fn decimal_decimal(self) -> Decimal {
    Decimal::from_parts(self.value.cast_unsigned(), 0, 0, self.value.is_negative(), 6)
  }

  pub const fn is_zero(self) -> bool {
    self.value == 0
  }

  #[expect(clippy::arithmetic_side_effects, reason = "constructors don't accept i32::MIN")]
  #[must_use]
  pub const fn neg(self) -> Self {
    Self { value: -self.value }
  }

  /// Integral percentage expressed as `f64`.
  pub const fn pct_f64(self) -> f64 {
    i32f64(self.value) / 10_000.0
  }

  /// Truncated percentage expressed as `i32`.
  pub const fn pct_i32(self) -> i32 {
    self.value / 10_000
  }

  /// Raw Parts Per Million value
  pub const fn ppm(self) -> i32 {
    self.value
  }

  #[must_use]
  pub const fn saturating_sub(self, other: Self) -> Self {
    Self { value: self.value.saturating_sub(other.value) }
  }
}

impl Debug for Ppm {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let string = i32_string(self.value);
    let (rest, twos) = string.as_bytes().as_rchunks::<2>();

    for elem in rest {
      f.write_char(char::from(*elem))?;
    }

    let mut iter = twos.iter();
    if let Some([b0, b1]) = iter.next() {
      if rest.last().is_some_and(|el| *el != b'-') {
        f.write_char('_')?;
      }
      f.write_char(char::from(*b0))?;
      f.write_char(char::from(*b1))?;
    }
    for [b0, b1] in iter {
      f.write_char('_')?;
      f.write_char(char::from(*b0))?;
      f.write_char(char::from(*b1))?;
    }
    f.write_str("ppm")?;

    Ok(())
  }
}

impl Display for Ppm {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

#[cfg(feature = "database")]
mod database {
  use crate::{
    codec::{Decode, Encode},
    database::{
      Typed,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
    },
    misc::Ppm,
  };

  impl<'de, E> Decode<'de, Postgres<E>> for Ppm
  where
    E: From<crate::Error>,
  {
    fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
      let value: i32 = Decode::<'_, Postgres<E>>::decode(dw)?;
      Ok(Self::from_ppm_i32(value)?)
    }
  }

  impl<E> Encode<Postgres<E>> for Ppm
  where
    E: From<crate::Error>,
  {
    fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      <i32 as Encode<Postgres<E>>>::encode(&self.value, ew)
    }
  }

  impl<E> Typed<Postgres<E>> for Ppm
  where
    E: From<crate::Error>,
  {
    fn runtime_ty(&self) -> Option<Ty> {
      None
    }

    fn static_ty() -> Option<Ty> {
      None
    }
  }
}

#[cfg(all(feature = "rust_decimal", test))]
mod tests {
  use crate::misc::Ppm;
  use rust_decimal::Decimal;

  #[test]
  fn constructors_convert_to_correct_values() {
    let _0_0025 = Decimal::from_parts(25, 0, 0, false, 4);

    let ppm = Ppm::from_decimal_decimal(_0_0025).unwrap();
    assert_eq!(ppm.decimal_decimal(), _0_0025);
    assert_eq!(ppm.ppm(), 2500);
    let ppm = Ppm::from_bps_i16(25);
    assert_eq!(ppm.decimal_decimal(), _0_0025);
    assert_eq!(ppm.ppm(), 2500);
    let ppm = Ppm::ONE_MILLION;
    assert_eq!(ppm.decimal_decimal(), Decimal::ONE);
    assert_eq!(ppm.ppm(), 1000000);
  }

  #[test]
  fn zero_is_a_valid_contructor_value() {
    assert!(Ppm::from_decimal_decimal(Decimal::ZERO).is_ok());
    assert!(Ppm::from_ppm_i32(0).is_ok());
  }
}
