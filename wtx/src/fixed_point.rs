//! Fixed decimal numbers

mod fixed_point_num;
mod i_ppm32;
mod multipliers;
mod types;

pub use fixed_point_num::FixedPointNum;
pub use i_ppm32::IPpm32;
pub use types::*;

const MAX_DECIMALS: usize = 10; // ppt

/// A generic decimal number where the fixed decimals are represented by `N` and the underlying
/// capacity is represented by `T`
#[derive(Debug)]
#[repr(transparent)]
pub struct FixedPoint<T, const N: usize>(T);

impl<T, const N: usize> FixedPoint<T, N>
where
  T: FixedPointNum,
{
  /// Instance with the minimum allowed value.
  pub const MIN: Self = Self(T::MIN);
  /// Instance with the maximum allowed value.
  pub const MAX: Self = Self(T::MAX);

  /// From a basis point expressed as `i16`.
  ///
  /// For example, 1bps becomes `0.0001`.
  #[inline]
  pub fn from_bps_i16(value: i16) -> Self
  where
    T: From<i32>,
  {
    const {
      assert!(i16::MIN_I128.checked_mul(Self::MULTIPLIER_BPS_I128).unwrap() >= T::MIN_I128);
      assert!(i16::MAX_U128.checked_mul(Self::MULTIPLIER_BPS_U128).unwrap() <= T::MAX_U128);
    }
    Self(T::from(i32::from(value).wrapping_mul(Self::MULTIPLIER_BPS_I32)))
  }

  /// From a basis point expressed as `u16`.
  ///
  /// For example, 1bps becomes `0.0001`.
  #[inline]
  pub fn from_bps_u16(value: u16) -> Self
  where
    T: From<u32>,
  {
    const {
      assert!(u16::MIN_I128.checked_mul(Self::MULTIPLIER_BPS_I128).unwrap() >= T::MIN_I128);
      assert!(u16::MAX_U128.checked_mul(Self::MULTIPLIER_BPS_U128).unwrap() <= T::MAX_U128);
    }
    Self(T::from(u32::from(value).wrapping_mul(Self::MULTIPLIER_BPS_U32)))
  }

  /// From a percentage expressed as `u8`.
  ///
  /// For example, 1% becomes `0.01`.
  #[inline]
  pub fn from_pct_u8(value: u8) -> Self
  where
    T: From<u32>,
  {
    const {
      assert!(u8::MIN_I128.checked_mul(Self::MULTIPLIER_PCT_I128).unwrap() >= T::MIN_I128);
      assert!(u8::MAX_U128.checked_mul(Self::MULTIPLIER_PCT_U128).unwrap() <= T::MAX_U128);
    };
    Self(T::from(u32::from(value).wrapping_mul(Self::MULTIPLIER_PCT_U32)))
  }

  /// From a percentage expressed as `i16`.
  ///
  /// For example, 1% becomes `0.01`.
  #[inline]
  pub fn from_pct_i16(value: i16) -> Self
  where
    T: From<i32>,
  {
    const {
      assert!(i16::MIN_I128.checked_mul(Self::MULTIPLIER_PCT_I128).unwrap() >= T::MIN_I128);
      assert!(i16::MAX_U128.checked_mul(Self::MULTIPLIER_PCT_U128).unwrap() <= T::MAX_U128);
    }
    Self(T::from(i32::from(value).wrapping_mul(Self::MULTIPLIER_PCT_I32)))
  }
}
