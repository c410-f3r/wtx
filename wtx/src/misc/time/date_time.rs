use crate::misc::{ArrayString, Date, Time};
use core::fmt::{Debug, Display, Formatter};

/// ISO-8601 representation with a fixed UTC timezone.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime {
  date: Date,
  time: Time,
}

impl DateTime {
  /// New instance from basic parameters
  #[inline]
  pub const fn new(date: Date, time: Time) -> Self {
    Self { date, time }
  }

  #[inline]
  fn to_str(self) -> ArrayString<24> {
    let mut rslt = ArrayString::new();
    let _rslt0 = rslt.push_str(&self.date.to_str());
    let _rslt1 = rslt.push('T');
    let _rslt2 = rslt.push_str(&self.time.to_str());
    let _rslt3 = rslt.push('Z');
    rslt
  }
}

impl Debug for DateTime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

impl Display for DateTime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::{DOY, Date, DateTime, Hour, Sixty, Time};

  fn _2025_04_20_14_20_30() -> DateTime {
    DateTime::new(Date::new(2025, DOY::N110), Time::from_hms(Hour::N14, Sixty::N20, Sixty::N30))
  }

  #[test]
  fn to_str() {
    assert_eq!(_2025_04_20_14_20_30().to_str().as_str(), "2025-04-20T14:20:30.0Z");
  }
}
