use crate::time::{
  Microsecond, Millisecond, NANOSECONDS_PER_MICROSECONDS, NANOSECONDS_PER_MILLISECOND, TimeError,
};
use core::hint;

/// This particular structure can represent at most one second in nanoseconds.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nanosecond(u32);

impl Nanosecond {
  /// Instance with the maximum allowed value of `999_999_999`
  pub const MAX: Self = Self(999_999_999);
  /// Instance with the minimum allowed value of `0`
  pub const ZERO: Self = Self(0);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u32) -> Result<Self, TimeError> {
    if num > 999_999_999 {
      return Err(TimeError::InvalidNanosecond { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u32 {
    self.0
  }

  /// Converts to the number of milliseconds
  #[inline]
  pub const fn to_ms(self) -> Millisecond {
    match Millisecond::from_num((self.0 / NANOSECONDS_PER_MILLISECOND) as u16) {
      Ok(elem) => elem,
      Err(_err) => {
        // SAFETY: The maximum value of this instance divided by `NANOSECONDS_PER_MICROSECONDS`
        // will never overflow the maximum allowed value.
        unsafe { hint::unreachable_unchecked() }
      }
    }
  }

  /// Converts to the number of microseconds
  #[inline]
  pub const fn to_us(self) -> Microsecond {
    match Microsecond::from_num(self.0 / NANOSECONDS_PER_MICROSECONDS) {
      Ok(elem) => elem,
      Err(_err) => {
        // SAFETY: The maximum value of this instance divided by `NANOSECONDS_PER_MICROSECONDS`
        // will never overflow the maximum allowed value.
        unsafe { hint::unreachable_unchecked() }
      }
    }
  }
}

impl TryFrom<u32> for Nanosecond {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u32) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
