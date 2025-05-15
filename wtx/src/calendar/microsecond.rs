use crate::calendar::{CalendarError, NANOSECONDS_PER_MICROSECONDS, Nanosecond};
use core::hint::unreachable_unchecked;

/// This particular structure can represent at most one second in microseconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Microsecond(u32);

impl Microsecond {
  /// Instance with the maximum allowed value of `999_999`
  pub const MAX: Self = Self(999_999);
  /// Instance with the minimum allowed value of `0`
  pub const ZERO: Self = Self(0);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u32) -> Result<Self, CalendarError> {
    if num > 999_999 {
      return Err(CalendarError::InvalidMicrosecond { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u32 {
    self.0
  }

  pub(crate) const fn to_ns(self) -> Nanosecond {
    match Nanosecond::from_num(self.0.wrapping_mul(NANOSECONDS_PER_MICROSECONDS)) {
      Ok(elem) => elem,
      // SAFETY: 999_999 * 1000 will never overflow the maximum allowed value of a nanosecond
      Err(_err) => unsafe { unreachable_unchecked() },
    }
  }
}

impl TryFrom<u32> for Microsecond {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u32) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
