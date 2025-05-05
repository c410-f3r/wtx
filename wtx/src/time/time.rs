use crate::{
  collection::ArrayString,
  misc::u32_string,
  time::{
    Hour, Microsecond, Minute, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, Second,
    misc::{u8u32, u16u32},
    nanosecond::Nanosecond,
  },
};
use core::{
  fmt::{Debug, Display, Formatter},
  hint,
};

/// Clock time with nanosecond precision.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Time {
  // | xxxxxx  | xxxxxx  | xxxxx |
  // | second  | minute  | hour  |
  params: u32,
  nanosecond: Nanosecond,
}

impl Time {
  /// Instance with the maximum allowed value of `23:59:59.999_999_999`
  pub const MAX: Self = Self::from_hms_ns(Hour::N23, Minute::N59, Second::N59, Nanosecond::MAX);
  /// Instance with the minimum allowed value of `00:00:00.000_000_000`
  pub const MIN: Self = Self::from_hms_ns(Hour::N0, Minute::N0, Second::N0, Nanosecond::ZERO);

  /// New instance without nanoseconds precision.
  #[inline]
  pub const fn from_hms(hour: Hour, minute: Minute, second: Second) -> Self {
    let mut params = u8u32(second.num()) << 11;
    params |= u8u32(minute.num()) << 5;
    params |= u8u32(hour.num());
    Self { params, nanosecond: Nanosecond::ZERO }
  }

  /// New instance with nanoseconds precision.
  #[inline]
  pub const fn from_hms_ns(
    hour: Hour,
    minute: Minute,
    second: Second,
    nanosecond: Nanosecond,
  ) -> Self {
    let mut this = Self::from_hms(hour, minute, second);
    this.nanosecond = nanosecond;
    this
  }

  /// New instance with microseconds precision.
  #[inline]
  pub const fn from_hms_us(
    hour: Hour,
    minute: Minute,
    second: Second,
    microsecond: Microsecond,
  ) -> Self {
    Self::from_hms_ns(hour, minute, second, microsecond.to_ns())
  }

  /// Hours of a day
  #[inline]
  pub const fn hour(self) -> Hour {
    match Hour::from_num((self.params & 0b1_1111) as u8) {
      Ok(el) => el,
      // SAFETY: All methods that create an instance only accept `Hour`, as such, the
      // corresponding bits will never be out of bounds.
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Minutes of a hour.
  #[inline]
  pub const fn minute(self) -> Minute {
    match Minute::from_num(((self.params >> 5) & 0b11_1111) as u8) {
      Ok(el) => el,
      // SAFETY: All methods that create an instance only accept `Minute`, as such, the
      // corresponding bits will never be out of bounds.
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Nanosecond of a second
  #[inline]
  pub const fn nanosecond(self) -> Nanosecond {
    self.nanosecond
  }

  /// Seconds of a minute
  #[inline]
  pub const fn second(self) -> Second {
    match Second::from_num(((self.params >> 11) & 0b11_1111) as u8) {
      Ok(el) => el,
      // SAFETY: All methods that create an instance only accept `Second`, as such, the
      // corresponding bits will never be out of bounds.
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// The total number of seconds since midnight (00:00:00).
  #[inline]
  pub const fn seconds_from_mn(self) -> u32 {
    let mut rslt = (u8u32(self.hour().num())).wrapping_mul(u16u32(SECONDS_PER_HOUR));
    rslt = rslt.wrapping_add(u8u32(self.minute().num()).wrapping_mul(u8u32(SECONDS_PER_MINUTE)));
    rslt.wrapping_add(u8u32(self.second().num()))
  }

  /// String representation
  #[inline]
  pub fn to_str(self) -> ArrayString<18> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(self.hour().num_str());
    let _rslt1 = array.push(':');
    let _rslt2 = array.push_str(self.minute().num_str());
    let _rslt3 = array.push(':');
    let _rslt4 = array.push_str(self.second().num_str());
    let nanosecond = self.nanosecond();
    if nanosecond.num() > 0 {
      let _rslt5 = array.push('.');
      let _rslt6 = array.push_str(&u32_string(nanosecond.num()));
    }
    array
  }
}

impl Debug for Time {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

impl Default for Time {
  #[inline]
  fn default() -> Self {
    Self::MIN
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
  use crate::time::{Hour, Minute, Second, Time, nanosecond::Nanosecond};

  fn _8_48_05_234_445_009() -> Time {
    Time::from_hms_ns(Hour::N8, Minute::N48, Second::N5, Nanosecond::from_num(234_445_009).unwrap())
  }

  fn _14_20_30() -> Time {
    Time::from_hms(Hour::N14, Minute::N20, Second::N30)
  }

  #[test]
  fn hour() {
    assert_eq!(Time::MIN.hour().num(), 0);
    assert_eq!(Time::MAX.hour().num(), 23);
    assert_eq!(_8_48_05_234_445_009().hour().num(), 8);
    assert_eq!(_14_20_30().hour().num(), 14);
  }

  #[test]
  fn minute() {
    assert_eq!(Time::MIN.minute().num(), 0);
    assert_eq!(Time::MAX.minute().num(), 59);
    assert_eq!(_8_48_05_234_445_009().minute().num(), 48);
    assert_eq!(_14_20_30().minute().num(), 20);
  }

  #[test]
  fn nanosecond() {
    assert_eq!(Time::MIN.nanosecond().num(), 0);
    assert_eq!(Time::MAX.nanosecond().num(), 999_999_999);
    assert_eq!(_8_48_05_234_445_009().nanosecond().num(), 234_445_009);
    assert_eq!(_14_20_30().nanosecond().num(), 0);
  }

  #[test]
  fn second() {
    assert_eq!(Time::MIN.second().num(), 0);
    assert_eq!(Time::MAX.second().num(), 59);
    assert_eq!(_8_48_05_234_445_009().second().num(), 5);
    assert_eq!(_14_20_30().second().num(), 30);
  }

  #[test]
  fn seconds_from_mn() {
    assert_eq!(Time::MIN.seconds_from_mn(), 0);
    assert_eq!(Time::MAX.seconds_from_mn(), 86_399);
    assert_eq!(_8_48_05_234_445_009().seconds_from_mn(), 28800 + 2880 + 5);
    assert_eq!(_14_20_30().seconds_from_mn(), 50400 + 1200 + 30);
  }

  #[test]
  fn to_str() {
    assert_eq!(Time::MIN.to_str().as_str(), "00:00:00");
    assert_eq!(Time::MAX.to_str().as_str(), "23:59:59.999999999");
    assert_eq!(_8_48_05_234_445_009().to_str().as_str(), "08:48:05.234445009");
    assert_eq!(_14_20_30().to_str().as_str(), "14:20:30");
  }
}
