use crate::{
  collection::ArrayString,
  misc::u32_string,
  time::{
    Hour, Minute, NANOSECONDS_PER_MILLISECOND, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, Second,
    misc::{u8u32, u8u64, u16u32, u32u64},
    nanosecond::Nanosecond,
  },
};
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
  /// Instance with the maximum allowed value of `23:59:59.999_999_999`
  pub const MAX: Self = Self::from_hms_ns(Hour::N23, Minute::N59, Second::N59, Nanosecond::MAX);
  /// Instance with the minimum allowed value of `00:00:00.000_000_000`
  pub const MIN: Self = Self::from_hms_ns(Hour::N0, Minute::N0, Second::N0, Nanosecond::MIN);

  /// New instance without nanoseconds precision.
  #[inline]
  pub const fn from_hms(hour: Hour, minute: Minute, second: Second) -> Self {
    let mut params = u8u64(second.num()) << 11;
    params |= u8u64(minute.num()) << 5;
    params |= u8u64(hour.num());
    Self { params }
  }

  /// New instance with nanoseconds precision.
  #[inline]
  pub const fn from_hms_ns(
    hour: Hour,
    minute: Minute,
    second: Second,
    nanosecond: Nanosecond,
  ) -> Self {
    let millisecond = nanosecond.num() / NANOSECONDS_PER_MILLISECOND;
    let mut params = u32u64(nanosecond.num()) << 27;
    params |= u32u64(millisecond) << 17;
    params |= u8u64(second.num()) << 11;
    params |= u8u64(minute.num()) << 5;
    params |= u8u64(hour.num());
    Self { params }
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

  /// Milliseconds of a second
  #[inline]
  pub const fn millisecond(self) -> u16 {
    ((self.params >> 17) & 0b11_1111_1111) as u16
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
  #[allow(clippy::cast_possible_truncation, reason = "The shift yields u32")]
  #[inline]
  pub const fn nanosecond(self) -> u32 {
    (self.params >> 27) as u32
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
    if nanosecond > 0 {
      let _rslt5 = array.push('.');
      let _rslt6 = array.push_str(&u32_string(nanosecond));
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
  fn millisecond() {
    assert_eq!(Time::MIN.millisecond(), 0);
    assert_eq!(Time::MAX.millisecond(), 999);
    assert_eq!(_8_48_05_234_445_009().millisecond(), 234);
    assert_eq!(_14_20_30().millisecond(), 0);
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
    assert_eq!(Time::MIN.nanosecond(), 0);
    assert_eq!(Time::MAX.nanosecond(), 999_999_999);
    assert_eq!(_8_48_05_234_445_009().nanosecond(), 234_445_009);
    assert_eq!(_14_20_30().nanosecond(), 0);
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
