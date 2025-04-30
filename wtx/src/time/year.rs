use crate::time::TimeError;

/// All possible years that can be represented by the system. Goes from -32767 to 32766.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Year(i16);

impl Year {
  /// Instance that refers the UNIX epoch (1970).
  pub const EPOCH: Self = Self(1970);
  /// Instance with the maximum allowed value of `32766`
  pub const MAX: Self = Self(32766);
  /// Instance with the minimum allowed value of `-32767`
  pub const MIN: Self = Self(-32767);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: i16) -> Result<Self, TimeError> {
    if num < -32767 || num > 32766 {
      return Err(TimeError::InvalidYear { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> i16 {
    self.0
  }
}
