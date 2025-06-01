#![allow(
  clippy::cast_possible_truncation,
  reason = "shifted integers are reduced to a smaller representation"
)]

mod format;
#[cfg(test)]
mod tests;

use crate::{
  calendar::{
    CalendarError, CalendarToken, CeDays, DAYS_OF_MONTHS, DAYS_PER_4_YEARS, DAYS_PER_NON_LEAP_YEAR,
    DAYS_PER_QUADCENTURY, Day, DayOfYear, Duration, Month, SECONDS_PER_DAY, Weekday,
    YEARS_PER_QUADCENTURY, Year,
    misc::{
      boolu16, boolu32, boolusize, i16i32, i32i64, u8i16, u8i32, u8u16, u8u32, u8usize, u16i32,
      u16u32, u32i64,
    },
  },
  collection::ArrayString,
  misc::{Usize, i16_string},
};
use core::{
  cmp::Ordering,
  fmt::{Debug, Display, Formatter},
  hash::{Hash, Hasher},
  hint::unreachable_unchecked,
};

// 401 because of `146_097 / 400`.
static QUADCENTURY_ADJUSTMENTS: &[u8; 401] = &[
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
#[derive(Clone, Copy)]
pub struct Date {
  year: Year,
  // | xxxxxx | x            | xxxxxxxxx       |
  // | unused | is leap year | day of the year |
  params: u16,
}

impl Date {
  /// Instance that refers the common era (0001-01-01).
  pub const CE: Self = if let Ok(elem) = Self::new(Year::CE, DayOfYear::ONE) {
    elem
  } else {
    panic!();
  };
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = if let Ok(elem) = Self::new(Year::EPOCH, DayOfYear::ONE) {
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
  pub const MIN: Self = if let Ok(elem) = Self::new(Year::MIN, DayOfYear::ONE) {
    elem
  } else {
    panic!();
  };

  /// Creates a new instance from the number of days since the common era (0001-01-01).
  #[inline]
  pub const fn from_ce_days(cd: CeDays) -> Result<Self, CalendarError> {
    let Some(days_plus_year) = cd.num().checked_add(u16i32(DAYS_PER_NON_LEAP_YEAR)) else {
      // SAFETY: `CeDays` has a upper bound way lower than the maximum capacity
      unsafe { unreachable_unchecked() }
    };
    let quadcenturies = days_plus_year.div_euclid(DAYS_PER_QUADCENTURY.cast_signed()) as i16;
    let quadcentury_days = days_plus_year.rem_euclid(DAYS_PER_QUADCENTURY.cast_signed());
    let Some((quadcentury_years, day_of_year)) = years_from_quadcentury_days(quadcentury_days)
    else {
      return Ok(Self::EPOCH);
    };
    let years = quadcenturies
      .wrapping_mul(YEARS_PER_QUADCENTURY.cast_signed())
      .wrapping_add(quadcentury_years);
    Self::new(
      match Year::from_num(years) {
        Ok(elem) => elem,
        Err(err) => return Err(err),
      },
      day_of_year,
    )
  }

  /// Creates a new instance based on the string representation of the ISO-8601 specification.
  #[inline]
  pub fn from_iso_8601(bytes: &[u8]) -> crate::Result<Self> {
    static TOKENS: &[CalendarToken] = &[
      CalendarToken::FourDigitYear,
      CalendarToken::Dash,
      CalendarToken::TwoDigitMonth,
      CalendarToken::Dash,
      CalendarToken::TwoDigitDay,
    ];
    Self::parse(bytes, TOKENS.iter().copied())
  }

  /// Constructs a new instance that automatically deals with leap years.
  #[inline]
  pub const fn from_ymd(year: Year, month: Month, day: Day) -> Result<Self, CalendarError> {
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
  pub const fn new(year: Year, day_of_year: DayOfYear) -> Result<Self, CalendarError> {
    let is_leap_year = year.is_leap_year();
    if !is_leap_year && day_of_year.num() == DayOfYear::N366.num() {
      return Err(CalendarError::InvalidDayOfTheYearInNonLeapYear);
    }
    let mut params = boolu16(is_leap_year) << 15;
    params |= day_of_year.num();
    Ok(Self { year, params })
  }

  /// Adds the number of whole days in the given `duration` to the current date.
  pub const fn add(self, duration: Duration) -> Result<Self, CalendarError> {
    if duration.is_zero() {
      return Ok(self);
    }
    let days = duration.seconds() / u32i64(SECONDS_PER_DAY);
    if days < i32i64(i32::MIN) || days > i32i64(i32::MAX) {
      return Err(CalendarError::ArithmeticOverflow);
    }
    self.add_days(days as i32)
  }

  /// Number of days since the common era (0001-01-01)
  #[inline]
  pub const fn ce_days(self) -> i32 {
    let mut year = i16i32(self.year().num().wrapping_sub(1));
    let mut days: i32 = 0;
    if year < 0 {
      let adjustment = (year.abs() / 400).wrapping_add(1);
      year = year.wrapping_add(adjustment.wrapping_mul(u16i32(YEARS_PER_QUADCENTURY)));
      days = days.wrapping_sub(adjustment.wrapping_mul(DAYS_PER_QUADCENTURY.cast_signed()));
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
      Err(_) => unsafe { unreachable_unchecked() },
    }
  }

  /// Day of the year.
  #[inline]
  pub const fn day_of_year(self) -> DayOfYear {
    match DayOfYear::from_num(self.params & 0b1_1111_1111) {
      Ok(el) => el,
      // SAFETY: All methods that create an instance only accept `DOY`, as such, the
      // corresponding bits will never be out of bounds.
      Err(_) => unsafe { unreachable_unchecked() },
    }
  }

  /// String representation
  #[inline]
  pub fn iso_8601(self) -> ArrayString<12> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(&i16_string(self.year.num()));
    let _rslt1 = array.push('-');
    let _rslt2 = array.push_str(self.month().num_str());
    let _rslt3 = array.push('-');
    let _rslt4 = array.push_str(self.day().num_str());
    array
  }

  /// Month of the year
  #[inline]
  pub const fn month(self) -> Month {
    let (_, month_helper, month_surplus) = self.month_params();
    let rslt = month_helper.wrapping_add(month_surplus);
    match Month::from_num(rslt) {
      Ok(el) => el,
      // SAFETY: `rslt` is always within the 1-12 range
      Err(_) => unsafe { unreachable_unchecked() },
    }
  }

  /// Subtracts the number of whole days in the given `duration` to the current date.
  pub const fn sub(self, duration: Duration) -> Result<Self, CalendarError> {
    let days = -duration.days();
    if days < i32i64(i32::MIN) || days > i32i64(i32::MAX) {
      return Err(CalendarError::ArithmeticOverflow);
    }
    self.add_days(days as i32)
  }

  /// Day of week.
  #[inline]
  pub const fn weekday(self) -> Weekday {
    match self.ce_days() % 7 {
      -6 | 1 => Weekday::Monday,
      -5 | 2 => Weekday::Tuesday,
      -4 | 3 => Weekday::Wednesday,
      -3 | 4 => Weekday::Thursday,
      -2 | 5 => Weekday::Friday,
      -1 | 6 => Weekday::Saturday,
      _ => Weekday::Sunday,
    }
  }

  /// Year
  #[inline]
  pub const fn year(self) -> Year {
    self.year
  }

  pub(crate) const fn add_days(self, days: i32) -> Result<Self, CalendarError> {
    let this_year = self.year().num();
    let this_quadcenturies = this_year.div_euclid(YEARS_PER_QUADCENTURY.cast_signed());
    let this_quadcentury_years = this_year.rem_euclid(YEARS_PER_QUADCENTURY.cast_signed());
    let Some(this_quadcentury_days) =
      days_from_quadcentury_years(this_quadcentury_years, self.day_of_year().num())
    else {
      return Ok(Self::EPOCH);
    };

    let Some(sum_days) = this_quadcentury_days.checked_add(days) else {
      return Err(CalendarError::ArithmeticOverflow);
    };
    let sum_quadcenturies = sum_days.div_euclid(DAYS_PER_QUADCENTURY.cast_signed()) as i16;
    let sum_quadcentury_days = sum_days.rem_euclid(DAYS_PER_QUADCENTURY.cast_signed());
    let Some((sum_quadcentury_years, day_of_year)) =
      years_from_quadcentury_days(sum_quadcentury_days)
    else {
      return Ok(Self::EPOCH);
    };

    let year_num = this_quadcenturies
      .wrapping_add(sum_quadcenturies)
      .saturating_mul(YEARS_PER_QUADCENTURY.cast_signed())
      .saturating_add(sum_quadcentury_years);
    let year = match Year::from_num(year_num) {
      Ok(elem) => elem,
      Err(_err) => return Err(CalendarError::ArithmeticOverflow),
    };
    Ok(match Self::new(year, day_of_year) {
      Ok(elem) => elem,
      // SAFETY: Leap years were already handled
      Err(_) => unsafe { unreachable_unchecked() },
    })
  }

  const fn as_i32(self) -> i32 {
    (i16i32(self.year.num()) << 16) | u16i32(self.params)
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
    f.write_str(&self.iso_8601())
  }
}

impl Default for Date {
  #[inline]
  fn default() -> Self {
    Self::EPOCH
  }
}

impl Display for Date {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso_8601())
  }
}

impl Eq for Date {}

impl Hash for Date {
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    self.as_i32().hash(state);
  }
}

impl Ord for Date {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_i32().cmp(&other.as_i32())
  }
}

impl PartialEq for Date {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.as_i32() == other.as_i32()
  }
}

impl PartialOrd for Date {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[allow(
  clippy::indexing_slicing,
  reason = "`years` is not greater than the length of `QUADCENTURY_ADJUSTMENTS`"
)]
const fn days_from_quadcentury_years(years: i16, day_of_year: u16) -> Option<i32> {
  if years > YEARS_PER_QUADCENTURY.cast_signed() {
    return None;
  }
  let idx = Usize::from_u16(years.cast_unsigned()).into_usize();
  Some(
    i16i32(years)
      .wrapping_mul(u16i32(DAYS_PER_NON_LEAP_YEAR))
      .wrapping_add(u8i32(QUADCENTURY_ADJUSTMENTS[idx]))
      .wrapping_add(u16i32(day_of_year))
      .wrapping_sub(1),
  )
}

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Divisions/modulos are using non-zero numbers but it can't see past a literal constant"
)]
#[allow(
  clippy::indexing_slicing,
  reason = "days / DAYS_PER_NON_LEAP_YEAR_U16 will never be greater than 400"
)]
const fn years_from_quadcentury_days(days: i32) -> Option<(i16, DayOfYear)> {
  if days > DAYS_PER_QUADCENTURY.cast_signed() {
    return None;
  }
  let mut year = (days / u16i32(DAYS_PER_NON_LEAP_YEAR)) as i16;
  let mut day_of_year = (days % u16i32(DAYS_PER_NON_LEAP_YEAR)) as i16;
  let idx = Usize::from_u16(year.unsigned_abs()).into_usize();
  let adjustment = u8i16(QUADCENTURY_ADJUSTMENTS[idx]);
  if day_of_year < adjustment {
    year = year.wrapping_sub(1);
    let local_idx = Usize::from_u16(year.unsigned_abs()).into_usize();
    let local_adjustment = u8i16(QUADCENTURY_ADJUSTMENTS[local_idx]);
    let value = DAYS_PER_NON_LEAP_YEAR.cast_signed().wrapping_sub(local_adjustment);
    day_of_year = day_of_year.wrapping_add(value);
  } else {
    day_of_year = day_of_year.wrapping_sub(adjustment);
  }
  Some((
    year,
    match DayOfYear::from_num(day_of_year.wrapping_add(1).unsigned_abs()) {
      Ok(elem) => elem,
      // SAFETY: value will never be greater than 366
      Err(_) => unsafe { unreachable_unchecked() },
    },
  ))
}

#[cfg(feature = "serde")]
mod serde {
  use crate::calendar::Date;
  use core::fmt;
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
  };

  impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor;

      impl Visitor<'_> for LocalVisitor {
        type Value = Date;

        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          formatter.write_str("a formatted date string")
        }

        #[inline]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          Date::from_iso_8601(value.as_bytes()).map_err(E::custom)
        }
      }

      deserializer.deserialize_str(LocalVisitor)
    }
  }

  impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(&self.iso_8601())
    }
  }
}
