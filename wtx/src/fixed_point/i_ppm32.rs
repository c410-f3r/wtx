use crate::{
  codec::{I32String, num_string},
  misc::{
    AsciiGraphic, const_ok,
    int_conv::{i8i32, i16i32, i32f64, u8i32},
  },
};
use core::fmt::{Debug, Display, Formatter};
#[cfg(feature = "rust_decimal")]
use rust_decimal::Decimal;

/// Signed Parts Per Million (PPM) composed by 32 bits.
///
/// * `1₁₀  = 10²%    = 10⁴bps = 10⁶ppm`
/// * `1ppm = 10⁻²bps = 10⁻⁴%  = 10⁻⁶₁₀`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IPpm32(i32);

impl IPpm32 {
  /// `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  pub const MAX: Self = Self(i32::MAX);
  /// `-2_147.483648₁₀` | `-214_748.3648%` | `-21_474_836.48bps` | `-2_147_483_648ppm`
  pub const MIN: Self = Self(i32::MIN);
  /// `1₁₀` | `100%` | `10_000bps` | `1_000_000ppm`
  pub const ONE_DECIMAL: Self = Self(1_000_000);
  /// `0₁₀` | `0%` | `0bps` | `0ppm`
  pub const ZERO: Self = Self(0);

  /// From `-3.2768₁₀` | `-327.68%` | `-32_768bps` | `-3_276_800ppm`
  /// To   `+3.2767₁₀` | `+327.67%` | `+32_767bps` | `+3_276_700ppm`
  #[inline]
  pub const fn from_bps_i16(value: i16) -> Self {
    Self(i16i32(value).wrapping_mul(100))
  }

  /// From `-2_147.483648₁₀` | `-214_748.3648%` | `-21_474_836.48bps` | `-2_147_483_648ppm`
  /// To   `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  #[inline]
  #[cfg(feature = "rust_decimal")]
  pub fn from_decimal_decimal(from: Decimal) -> crate::Result<Self> {
    const ONE_MILLION: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);
    let fun = || Some(Self::from_ppm_i32(i32::try_from(from.checked_mul(ONE_MILLION)?).ok()?));
    fun().ok_or(crate::Error::InvalidPpmValue)
  }

  /// From `-128₁₀` | `-12_800%` | `-1_280_000bps` | `-128_000_000ppm`
  /// To   `+127₁₀` | `+12_700%` | `+1_270_000bps` | `+127_000_000ppm`
  #[inline]
  pub const fn from_decimal_i8(from: i8) -> Self {
    Self(i8i32(from).wrapping_mul(1_000_000))
  }

  /// From `  0₁₀` |      `0%` |         `0bps` |           `0ppm`
  /// To   `255₁₀` | `25_500%` | `2_550_000bps` | `255_000_000ppm`
  #[inline]
  pub const fn from_decimal_u8(from: u8) -> Self {
    Self(u8i32(from).wrapping_mul(1_000_000))
  }

  /// Lossy conversion from a `f64` percentage
  ///
  /// From `-327.68₁₀` | `-32_768%` | `-3_276_800bps` | `-327_680_000ppm`
  /// To   `+327.67₁₀` | `+32_767%` | `+3_276_700bps` | `+327_670_000ppm`
  #[expect(clippy::as_conversions, clippy::cast_possible_truncation, reason = "purposefully lossy")]
  #[inline]
  pub const fn from_pct_f64(value: f64) -> crate::Result<Self> {
    let ppm = value * 10_000.0;
    Ok(Self(ppm as i32))
  }

  /// From `-327.68₁₀` | `-32_768%` | `-3_276_800bps` | `-327_680_000ppm`
  /// To   `+327.67₁₀` | `+32_767%` | `+3_276_700bps` | `+327_670_000ppm`
  #[inline]
  pub const fn from_pct_i16(value: i16) -> Self {
    Self(i16i32(value).wrapping_mul(10_000))
  }

  /// From `-0.032768₁₀` | `-3.2768%` | `-327.68bps` | `-32_768ppm`
  /// To   `+0.032767₁₀` | `+3.2767%` | `+327.67bps` | `+32_767ppm`
  #[inline]
  pub const fn from_ppm_i16(value: i16) -> Self {
    Self(i16i32(value))
  }

  /// From `-2_147.483648₁₀` | `-214_748.3648%` | `-21_474_836.48bps` | `-2_147_483_648ppm`
  /// To   `+2_147.483647₁₀` | `+214_748.3647%` | `+21_474_836.47bps` | `+2_147_483_647ppm`
  #[inline]
  pub const fn from_ppm_i32(value: i32) -> Self {
    Self(value)
  }

  /// For example, if Ppm is 2%, then its complement value is 98% in its decimal form
  #[inline]
  #[cfg(feature = "rust_decimal")]
  pub fn complement_decimal_decimal(self) -> Decimal {
    Decimal::ONE.saturating_sub(self.decimal_decimal())
  }

  /// Decimal form in `rust_decimal`
  #[inline]
  #[cfg(feature = "rust_decimal")]
  pub const fn decimal_decimal(self) -> Decimal {
    Decimal::from_parts(self.0.unsigned_abs(), 0, 0, self.0.is_negative(), 6)
  }

  /// Is zero?
  #[inline]
  pub const fn is_zero(self) -> bool {
    self.0 == 0
  }

  /// Integral percentage expressed as `f64`.
  #[inline]
  pub const fn pct_f64(self) -> f64 {
    i32f64(self.0) / 10_000.0
  }

  /// Truncated percentage expressed as `i32`.
  #[inline]
  pub const fn pct_i32(self) -> i32 {
    self.0 / 10_000
  }

  /// Raw Parts Per Million value
  #[inline]
  pub const fn ppm(self) -> i32 {
    self.0
  }

  /// Representation in decimal string with the given number of decimals
  #[inline]
  pub fn repr_decimal(&self, decimals: u8) -> I32String {
    let final_decimals = decimals.min(6);
    let surplus_decimals: i32 = 6u8.wrapping_sub(final_decimals).into();
    let surplus_divisor = 10i32.checked_pow(surplus_decimals.unsigned_abs()).unwrap_or_default();
    let final_value = self.0.checked_div(surplus_divisor).unwrap_or_default();
    num_string(
      0,
      Some((const { const_ok(AsciiGraphic::new(b'.')).unwrap() }, final_decimals)),
      final_value,
      0,
    )
  }

  /// Computes `self - rhs`, saturating at the numeric bounds instead of overflowing.
  #[inline]
  #[must_use]
  pub const fn saturating_sub(self, other: Self) -> Self {
    Self(self.0.saturating_sub(other.0))
  }
}

impl Debug for IPpm32 {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.repr_decimal(6))
  }
}

impl Display for IPpm32 {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.repr_decimal(6))
  }
}

#[cfg(feature = "database")]
mod database {
  use crate::{
    codec::{Decode, Encode},
    database::{Database, Typed},
    fixed_point::IPpm32,
  };

  impl<'de, DB> Decode<'de, DB> for IPpm32
  where
    DB: Database,
    i32: Decode<'de, DB>,
  {
    #[inline]
    fn decode(dw: &mut DB::DecodeWrapper<'de, '_, '_>) -> Result<Self, DB::Error> {
      let value: i32 = Decode::<'_, DB>::decode(dw)?;
      Ok(Self::from_ppm_i32(value))
    }
  }

  impl<DB> Encode<DB> for IPpm32
  where
    DB: Database,
    i32: Encode<DB>,
  {
    #[inline]
    fn encode(&self, ew: &mut DB::EncodeWrapper<'_, '_, '_>) -> Result<(), DB::Error> {
      <i32 as Encode<DB>>::encode(&self.0, ew)
    }
  }

  impl<DB> Typed<DB> for IPpm32
  where
    DB: Database,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<DB::Ty> {
      None
    }

    #[inline]
    fn static_ty() -> Option<DB::Ty> {
      None
    }
  }
}

#[cfg(all(feature = "rust_decimal", test))]
mod tests {
  use crate::fixed_point::IPpm32;
  use rust_decimal::Decimal;

  #[test]
  fn constructors_convert_to_correct_values() {
    let _0_0025 = Decimal::from_parts(25, 0, 0, false, 4);

    let ppm = IPpm32::from_decimal_decimal(_0_0025).unwrap();
    assert_eq!(ppm.decimal_decimal(), _0_0025);
    assert_eq!(ppm.ppm(), 2500);
    let ppm = IPpm32::from_bps_i16(25);
    assert_eq!(ppm.decimal_decimal(), _0_0025);
    assert_eq!(ppm.ppm(), 2500);
    let ppm = IPpm32::ONE_DECIMAL;
    assert_eq!(ppm.decimal_decimal(), Decimal::ONE);
    assert_eq!(ppm.ppm(), 1000000);
  }

  #[test]
  fn representations() {
    {
      let ppm = IPpm32::from_pct_i16(12345);
      assert_eq!(ppm.repr_decimal(2), "123.45");
      assert_eq!(ppm.repr_decimal(0), "123");
    }
    {
      let ppm = IPpm32::from_pct_i16(1);
      assert_eq!(ppm.repr_decimal(2), "0.01");
      assert_eq!(ppm.repr_decimal(0), "0");
    }
  }
}
