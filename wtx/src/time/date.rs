#![allow(
  clippy::cast_possible_truncation,
  reason = "shifted integers can and will be reduced to a lighter representation"
)]

use crate::{
  collection::ArrayString,
  misc::{Usize, i16_string},
  time::{
    CeDays, DAYS_OF_MONTHS, DAYS_PER_4_YEARS, DAYS_PER_400_YEARS_I32, DAYS_PER_NON_LEAP_YEAR_I16,
    DAYS_PER_NON_LEAP_YEAR_U16, Day, DayOfYear, Month, TimeError, Year,
    misc::{boolu16, boolu32, boolusize, i16i32, u8i16, u8u16, u8u32, u8usize, u16i32, u16u32},
  },
};
use core::{
  fmt::{Debug, Display, Formatter},
  hint,
};

// 401 because of `146_097 / 400`.
const QUADRICENTURY_ADJUSTMENTS: &[u8; 401] = &[
  0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
  8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
  15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20,
  21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25,
  26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30, 30, 31, 31, 31, 31,
  32, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 35, 35, 35, 35, 36, 36, 36, 36, 37, 37, 37, 37,
  38, 38, 38, 38, 39, 39, 39, 39, 40, 40, 40, 40, 41, 41, 41, 41, 42, 42, 42, 42, 43, 43, 43, 43,
  44, 44, 44, 44, 45, 45, 45, 45, 46, 46, 46, 46, 47, 47, 47, 47, 48, 48, 48, 48, 49, 49, 49, 49,
  49, 49, 49, 49, 50, 50, 50, 50, 51, 51, 51, 51, 52, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54, 54,
  55, 55, 55, 55, 56, 56, 56, 56, 57, 57, 57, 57, 58, 58, 58, 58, 59, 59, 59, 59, 60, 60, 60, 60,
  61, 61, 61, 61, 62, 62, 62, 62, 63, 63, 63, 63, 64, 64, 64, 64, 65, 65, 65, 65, 66, 66, 66, 66,
  67, 67, 67, 67, 68, 68, 68, 68, 69, 69, 69, 69, 70, 70, 70, 70, 71, 71, 71, 71, 72, 72, 72, 72,
  73, 73, 73, 73, 73, 73, 73, 73, 74, 74, 74, 74, 75, 75, 75, 75, 76, 76, 76, 76, 77, 77, 77, 77,
  78, 78, 78, 78, 79, 79, 79, 79, 80, 80, 80, 80, 81, 81, 81, 81, 82, 82, 82, 82, 83, 83, 83, 83,
  84, 84, 84, 84, 85, 85, 85, 85, 86, 86, 86, 86, 87, 87, 87, 87, 88, 88, 88, 88, 89, 89, 89, 89,
  90, 90, 90, 90, 91, 91, 91, 91, 92, 92, 92, 92, 93, 93, 93, 93, 94, 94, 94, 94, 95, 95, 95, 95,
  96, 96, 96, 96, 97, 97, 97, 97,
];

/// Proleptic Gregorian calendar.
///
/// Can represent years from -32767 to +32766.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Date {
  // | xxxxxx | x            | xxxxxxxxx       |
  // | unused | is leap year | day of the year |
  params: u16,
  year: Year,
}

impl Date {
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = if let Ok(elem) = Self::new(Year::EPOCH, DayOfYear::MIN) {
    elem
  } else {
    panic!();
  };
  /// Instance with the maximum allowed value of `32768-12-31`
  pub const MAX: Self = if let Ok(elem) = Self::new(Year::MAX, DayOfYear::N365) {
    elem
  } else {
    panic!();
  };
  /// Instance with the minimum allowed value of `-32767-01-01`
  pub const MIN: Self = if let Ok(elem) = Self::new(Year::MIN, DayOfYear::MIN) {
    elem
  } else {
    panic!();
  };

  /// Creates a new instance from the number of days since the common era (0001-01-01).
  #[inline]
  pub const fn from_ce_days(cd: CeDays) -> Result<Self, TimeError> {
    let Some(days_plus_year) = cd.num().checked_add(u16i32(DAYS_PER_NON_LEAP_YEAR_U16)) else {
      // SAFETY: `CeDays` has a upper bound way lower than the maximum capacity
      unsafe { hint::unreachable_unchecked() }
    };
    let quadricenturies = days_plus_year.div_euclid(DAYS_PER_400_YEARS_I32) as i16;
    let remaining_days = days_plus_year.rem_euclid(DAYS_PER_400_YEARS_I32);
    let Some((mut year, day_of_year)) = years_from_quadricentury_days(remaining_days) else {
      return Self::new(Year::MIN, DayOfYear::MIN);
    };
    year = quadricenturies.wrapping_mul(400).wrapping_add(year);
    Self::new(
      match Year::from_num(year) {
        Ok(elem) => elem,
        Err(err) => return Err(err),
      },
      day_of_year,
    )
  }

  /// Constructs a new instance that automatically deals with leap years.
  #[inline]
  pub const fn from_ymd(year: Year, month: Month, day: Day) -> Result<Self, TimeError> {
    #[allow(clippy::indexing_slicing, reason = "zero or one are valid indices for a 2 len array")]
    let months_year = &DAYS_OF_MONTHS[boolusize(year.is_leap_year())];
    #[allow(clippy::indexing_slicing, reason = "month only goes up to 12")]
    let month_days = months_year[u8usize(month.num()).wrapping_sub(1)];
    let day_of_year = match DayOfYear::from_num(month_days.wrapping_add(u8u16(day.num()))) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Self::new(year, day_of_year)
  }

  /// Constructs a new instance that automatically deals with leap years.
  #[inline]
  pub const fn new(year: Year, day_of_year: DayOfYear) -> Result<Self, TimeError> {
    let is_leap_year = year.is_leap_year();
    if !is_leap_year && day_of_year.num() == DayOfYear::MAX.num() {
      return Err(TimeError::InvalidDayOfTheYearInNonLeapYear);
    }
    let mut params = boolu16(is_leap_year) << 15;
    params |= day_of_year.num();
    Ok(Self { params, year })
  }

  /// Number of days since the common era (0001-01-01)
  #[inline]
  pub const fn ce_days(self) -> i32 {
    let mut year = i16i32(self.year().num().wrapping_sub(1));
    let mut days: i32 = 0;
    if year < 0 {
      let adjustment = (year.abs() / 400).wrapping_add(1);
      year = year.wrapping_add(adjustment.wrapping_mul(400));
      days = days.wrapping_sub(adjustment.wrapping_mul(DAYS_PER_400_YEARS_I32));
    }
    let centuries = year / 100;
    days = days.wrapping_add(year.wrapping_mul(u16i32(DAYS_PER_4_YEARS)) / 4);
    days = days.wrapping_add(centuries / 4);
    days = days.wrapping_sub(centuries);
    days.wrapping_add(u16i32(self.day_of_year().num()))
  }

  /// Day of the month.
  //
  // Credits to https://jhpratt.dev/blog/optimizing-with-novel-calendrical-algorithms.
  #[inline]
  pub const fn day(self) -> Day {
    let (day_of_year, month_helper, _) = self.month_params();
    let days_in_preceding_months = u8u32(month_helper).wrapping_mul(3917).wrapping_sub(3866) >> 7;
    let rslt = day_of_year.wrapping_sub(days_in_preceding_months) as u8;
    match Day::from_num(rslt) {
      Ok(el) => el,
      // SAFETY: `rslt` is always within the 1-31 range
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Day of the year.
  #[inline]
  pub const fn day_of_year(self) -> DayOfYear {
    match DayOfYear::from_num(self.params & 0b1_1111_1111) {
      Ok(el) => el,
      // SAFETY: All methods that create an instance only accept `DOY`, as such, the
      // corresponding bits will never be out of bounds.
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// Month of the year
  #[inline]
  pub const fn month(self) -> Month {
    let (_, month_helper, month_surplus) = self.month_params();
    let rslt = month_helper.wrapping_add(month_surplus);
    match Month::from_num(rslt) {
      Ok(el) => el,
      // SAFETY: `rslt` is always within the 1-12 range
      Err(_) => unsafe { hint::unreachable_unchecked() },
    }
  }

  /// String representation
  #[inline]
  pub fn to_str(self) -> ArrayString<12> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(&i16_string(self.year.num()));
    let _rslt1 = array.push('-');
    let _rslt2 = array.push_str(self.month().num_str());
    let _rslt3 = array.push('-');
    let _rslt4 = array.push_str(self.day().num_str());
    array
  }

  /// Year
  #[inline]
  pub const fn year(self) -> Year {
    self.year
  }

  // Credits to https://jhpratt.dev/blog/optimizing-with-novel-calendrical-algorithms.
  const fn month_params(self) -> (u32, u8, u8) {
    let days_until_feb = 59u32.wrapping_add(boolu32(self.year().is_leap_year()));
    let mut day_of_year = u16u32(self.day_of_year().num());
    let mut month_surplus = 0;
    if let Some(elem @ 1..=u32::MAX) = day_of_year.checked_sub(days_until_feb) {
      day_of_year = elem;
      month_surplus = 2;
    }
    let month_helper = day_of_year.wrapping_mul(268).wrapping_add(8031) >> 13;
    (day_of_year, month_helper as u8, month_surplus)
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

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Divisions/modulos are using non-zero numbers but it can't see past a literal constant"
)]
#[allow(
  clippy::indexing_slicing,
  reason = "days / DAYS_PER_NON_LEAP_YEAR_U16 will never be greater than 400"
)]
const fn years_from_quadricentury_days(days: i32) -> Option<(i16, DayOfYear)> {
  if days > DAYS_PER_400_YEARS_I32 {
    return None;
  }
  let mut year = (days / u16i32(DAYS_PER_NON_LEAP_YEAR_U16)) as i16;
  let mut day_of_year = (days % u16i32(DAYS_PER_NON_LEAP_YEAR_U16)) as i16;
  let idx = Usize::from_u16(year.unsigned_abs()).into_usize();
  let adjustment = u8i16(QUADRICENTURY_ADJUSTMENTS[idx]);
  if day_of_year < adjustment {
    year = year.wrapping_sub(1);
    let local_idx = Usize::from_u16(year.unsigned_abs()).into_usize();
    let local_adjustment = u8i16(QUADRICENTURY_ADJUSTMENTS[local_idx]);
    let value = DAYS_PER_NON_LEAP_YEAR_I16.wrapping_sub(local_adjustment);
    day_of_year = day_of_year.wrapping_add(value);
  } else {
    day_of_year = day_of_year.wrapping_sub(adjustment);
  }
  Some((
    year,
    match DayOfYear::from_num(day_of_year.wrapping_add(1).unsigned_abs()) {
      Ok(elem) => elem,
      // SAFETY: value will never be greater than 366
      Err(_) => unsafe { hint::unreachable_unchecked() },
    },
  ))
}

#[cfg(test)]
mod tests {
  use crate::time::{CeDays, DAYS_PER_400_YEARS_I32, Date, DayOfYear, Year};

  fn _0401_03_02() -> Date {
    Date::from_ce_days(CeDays::from_num(DAYS_PER_400_YEARS_I32 + 59 + 2).unwrap()).unwrap()
  }

  fn _2025_04_20() -> Date {
    Date::new(Year::from_num(2025).unwrap(), DayOfYear::from_num(110).unwrap()).unwrap()
  }

  #[test]
  fn ce_days() {
    assert_eq!(Date::MIN.ce_days(), -11968265);
    assert_eq!(Date::MAX.ce_days(), 11967535);
    assert_eq!(_0401_03_02().ce_days(), DAYS_PER_400_YEARS_I32 + 59 + 2);
    assert_eq!(_2025_04_20().ce_days(), 739361);
  }

  #[test]
  fn constructors_converge() {
    assert_eq!(
      Date::new(Year::from_num(500).unwrap(), DayOfYear::from_num(104).unwrap()).unwrap(),
      Date::from_ce_days(CeDays::from_num(182360).unwrap()).unwrap()
    );
  }

  #[test]
  fn day() {
    assert_eq!(Date::MIN.day().num(), 1);
    assert_eq!(Date::MAX.day().num(), 31);
    assert_eq!(_0401_03_02().day().num(), 02);
    assert_eq!(_2025_04_20().day().num(), 20);
  }

  #[test]
  fn day_of_year() {
    assert_eq!(Date::MIN.day_of_year().num(), 1);
    assert_eq!(Date::MAX.day_of_year().num(), 365);
    assert_eq!(_0401_03_02().day_of_year().num(), 61);
    assert_eq!(_2025_04_20().day_of_year().num(), 110);
  }

  #[test]
  fn month() {
    assert_eq!(Date::MIN.month().num(), 1);
    assert_eq!(Date::MAX.month().num(), 12);
    assert_eq!(_0401_03_02().month().num(), 3);
    assert_eq!(_2025_04_20().month().num(), 4);
  }

  #[test]
  fn to_str() {
    assert_eq!(Date::MIN.to_str().as_str(), "-32767-01-01");
    assert_eq!(Date::MAX.to_str().as_str(), "32766-12-31");
    assert_eq!(_0401_03_02().to_str().as_str(), "401-03-02");
    assert_eq!(_2025_04_20().to_str().as_str(), "2025-04-20");
  }

  #[test]
  fn year() {
    assert_eq!(Date::MIN.year().num(), -32767);
    assert_eq!(Date::MAX.year().num(), 32766);
    assert_eq!(_0401_03_02().year().num(), 401);
    assert_eq!(_2025_04_20().year().num(), 2025);
  }
}
