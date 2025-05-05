use crate::time::TimeError;

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
  pub const fn from_num(num: u16) -> Result<Self, TimeError> {
    if num > 999 {
      return Err(TimeError::InvalidMillisecond { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u16 {
    self.0
  }
}

impl TryFrom<u16> for Millisecond {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u16) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
