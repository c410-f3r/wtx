use crate::calendar::CalendarError;

/// All possible years that can be represented by the system. Goes from -32767 to 32766.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Year(i16);

impl Year {
  /// Instance that refers the common era (0001).
  pub const CE: Self = Self(1);
  /// Instance that refers the UNIX epoch (1970).
  pub const EPOCH: Self = Self(1970);
  /// Instance with the maximum allowed value of `32766`
  pub const MAX: Self = Self(32766);
  /// Instance with the minimum allowed value of `-32767`
  pub const MIN: Self = Self(-32767);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: i16) -> Result<Self, CalendarError> {
    if num < Self::MIN.num() || num > Self::MAX.num() {
      return Err(CalendarError::InvalidYear { received: num });
    }
    Ok(Self(num))
  }

  /// If this instance has an additional day.
  #[inline]
  pub const fn is_leap_year(&self) -> bool {
    let value = if self.0 % 100 == 0 { 0b1111 } else { 0b11 };
    self.0 & value == 0
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> i16 {
    self.0
  }
}

impl TryFrom<i16> for Year {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: i16) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
