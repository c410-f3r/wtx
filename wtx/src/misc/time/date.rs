#![allow(clippy::as_conversions, reason = "lack of constant evaluation for traits")]
#![allow(
  clippy::cast_possible_truncation,
  reason = "shifted integers can and will be reduced to a lighter representation"
)]

use crate::misc::{ArrayString, DOY, Day, Month, i16_string};
use core::{
  fmt::{Debug, Display, Formatter},
  hint,
};

/// Proleptic Gregorian calendar.
///
/// Can represent years from -32768 to +32768.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Date {
  // | xxxxxx | x            | xxxxxxxxx       |
  // | unused | is leap year | day of the year |
  params: u16,
  year: i16,
}

impl Date {
  /// Constructs a new instance that automatically deals with leap years.
  #[inline]
  pub const fn new(year: i16, doy: DOY) -> Self {
    let mut params = (is_leap_year(year) as u16) << 15;
    params |= doy.num();
    Self { params, year }
  }

  /// Day of the month.
  #[inline]
  pub const fn day(self) -> Day {
    let (doy, month_helper, _) = self.month_params();
    let days_in_preceding_months = (month_helper as u32).wrapping_mul(3917).wrapping_sub(3866) >> 7;
    let rslt = doy.wrapping_sub(days_in_preceding_months) as u8;
    match Day::from_num(rslt) {
      Some(el) => el,
      // SAFETY: `rslt` is always within the 1-31 range
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Day of the year.
  #[inline]
  pub const fn doy(self) -> DOY {
    match DOY::from_num(self.params & 0b1_1111_1111) {
      Some(el) => el,
      // SAFETY: All methods that create an instance only accept `DOY`, as such, the
      // corresponding bits will never be out of bounds.
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// If this instance has an additional day.
  #[inline]
  pub const fn is_leap_year(self) -> bool {
    self.params & 0b10_0000_0000 == 0b10_0000_0000
  }

  /// Month of the year
  #[inline]
  pub const fn month(self) -> Month {
    let (_, month_helper, month_surplus) = self.month_params();
    let rslt = month_helper.wrapping_add(month_surplus);
    match Month::from_num(rslt) {
      Some(el) => el,
      // SAFETY: `rslt` is always within the 1-12 range
      None => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Year
  #[inline]
  pub const fn year(self) -> i16 {
    self.year
  }

  // Credits to https://jhpratt.dev/blog/optimizing-with-novel-calendrical-algorithms.
  #[inline]
  const fn month_params(self) -> (u32, u8, u8) {
    let days_until_feb = 59u32.wrapping_add(self.is_leap_year() as u32);
    let mut doy = self.doy().num() as u32;
    let mut month_surplus = 0;
    if let Some(elem @ 1..=u32::MAX) = doy.checked_sub(days_until_feb) {
      doy = elem;
      month_surplus = 2;
    }
    let month_helper = doy.wrapping_mul(268).wrapping_add(8031) >> 13;
    (doy, month_helper as u8, month_surplus)
  }

  #[inline]
  pub(crate) fn to_str(self) -> ArrayString<10> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(&i16_string(self.year));
    let _rslt1 = array.push('-');
    let _rslt2 = array.push_str(self.month().num_str());
    let _rslt3 = array.push('-');
    let _rslt4 = array.push_str(self.day().num_str());
    array
  }
}

impl Debug for Date {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

impl Display for Date {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
  }
}

#[inline]
const fn is_leap_year(year: i16) -> bool {
  let value = if year % 100 == 0 { 15 } else { 3 };
  year & value == 0
}

#[cfg(test)]
mod tests {
  use crate::misc::{DOY, Date};

  fn _2025_04_20() -> Date {
    Date::new(2025, DOY::N110)
  }

  #[test]
  fn day() {
    assert_eq!(_2025_04_20().day().num(), 20);
  }

  #[test]
  fn doy() {
    assert_eq!(_2025_04_20().doy().num(), 110);
  }

  #[test]
  fn is_leap_year() {
    assert_eq!(_2025_04_20().is_leap_year(), false);
  }

  #[test]
  fn month() {
    assert_eq!(_2025_04_20().month().num(), 4);
  }

  #[test]
  fn to_str() {
    assert_eq!(_2025_04_20().to_str().as_str(), "2025-04-20");
  }

  #[test]
  fn year() {
    assert_eq!(_2025_04_20().year(), 2025);
  }
}
