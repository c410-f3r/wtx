use crate::calendar::{CalendarError, NANOSECONDS_PER_MILLISECOND, Nanosecond, misc::u16u32};
use core::hint::unreachable_unchecked;

/// This particular structure can represent at most one second in milliseconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Millisecond(u16);

impl Millisecond {
  /// Instance with the maximum allowed value of `999`
  pub const MAX: Self = Self(999);
  /// Instance with the minimum allowed value of `0`
  pub const ZERO: Self = Self(0);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u16) -> Result<Self, CalendarError> {
    if num > 999 {
      return Err(CalendarError::InvalidMillisecond { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u16 {
    self.0
  }

  pub(crate) const fn to_ns(self) -> Nanosecond {
    match Nanosecond::from_num(u16u32(self.0).wrapping_mul(NANOSECONDS_PER_MILLISECOND)) {
      Ok(elem) => elem,
      // SAFETY: 999 * 1_000_000 will never overflow the maximum allowed value of a nanosecond
      Err(_err) => unsafe { unreachable_unchecked() },
    }
  }
}

impl TryFrom<u16> for Millisecond {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u16) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
