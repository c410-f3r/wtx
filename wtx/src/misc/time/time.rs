#![allow(
  clippy::cast_possible_truncation,
  reason = "shifted integers can and will be reduced to a lighter representation"
)]
#![allow(clippy::as_conversions, reason = "lack of constant evaluation for traits")]

use crate::misc::{ArrayString, Hour, Sixty, u16_string};
use core::{
  fmt::{Debug, Display, Formatter},
  hint,
};

/// Clock time with nanosecond precision.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Time {
  // | xxxxxxx | xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx | xxxxxxxxxx   | xxxxxx  | xxxxxx  | xxxxx |
  // | unused  | nanosecond                     | millisecond  | second  | minute  | hour  |
  params: u64,
}

impl Time {
  /// New instance
  #[inline]
  pub const fn from_hms(hour: Hour, minute: Sixty, second: Sixty) -> Self {
    let mut params = (second.num() as u64) << 11;
    params |= (minute.num() as u64) << 5;
    params |= hour.num() as u64;
    Self { params }
  }

  /// Hours of a day
  #[inline]
  pub const fn hour(self) -> Hour {
    match Hour::from_num((self.params & 0b1_1111) as u8) {
      Some(el) => el,
      // SAFETY: All methods that create an instance only accept `Hour`, as such, the
      // corresponding bits will never be out of bounds.
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Millisecond
  #[inline]
  pub const fn millisecond(self) -> u16 {
    ((self.params >> 17) & 0b11_1111_1111) as u16
  }

  /// Minutes for a hour.
  #[inline]
  pub const fn minute(self) -> Sixty {
    match Sixty::from_num(((self.params >> 5) & 0b11_1111) as u8) {
      Some(el) => el,
      // SAFETY: All methods that create an instance only accept `Sixty`, as such, the
      // corresponding bits will never be out of bounds.
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Nanosecond
  #[inline]
  pub const fn nanosecond(self) -> u32 {
    ((self.params) >> 27) as u32
  }

  /// Seconds
  #[inline]
  pub const fn second(self) -> Sixty {
    match Sixty::from_num(((self.params >> 11) & 0b11_1111) as u8) {
      Some(el) => el,
      // SAFETY: All methods that create an instance only accept `Sixty`, as such, the
      // corresponding bits will never be out of bounds.
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  // HH:mm:ss.SSS
  #[inline]
  pub(crate) fn to_str(self) -> ArrayString<12> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(self.hour().num_str());
    let _rslt1 = array.push(':');
    let _rslt2 = array.push_str(self.minute().num_str());
    let _rslt3 = array.push(':');
    let _rslt4 = array.push_str(self.second().num_str());
    let _rslt5 = array.push('.');
    let _rslt6 = array.push_str(&u16_string(self.millisecond()));
    array
  }
}

impl Debug for Time {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

impl Display for Time {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::{Hour, Sixty, Time};

  fn _14_20_30() -> Time {
    Time::from_hms(Hour::N14, Sixty::N20, Sixty::N30)
  }

  #[test]
  fn hour() {
    assert_eq!(_14_20_30().hour().num(), 14);
  }

  #[test]
  fn millisecond() {
    assert_eq!(_14_20_30().millisecond(), 0);
  }

  #[test]
  fn minute() {
    assert_eq!(_14_20_30().minute().num(), 20);
  }

  #[test]
  fn nanosecond() {
    assert_eq!(_14_20_30().nanosecond(), 0);
  }

  #[test]
  fn second() {
    assert_eq!(_14_20_30().second().num(), 30);
  }

  #[test]
  fn to_str() {
    assert_eq!(_14_20_30().to_str().as_str(), "14:20:30.0");
  }
}
