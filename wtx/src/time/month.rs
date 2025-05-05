use crate::time::{TimeError, Year};

/// Month of the year.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Month {
  /// January
  January,
  /// February
  February,
  /// March
  March,
  /// April
  April,
  /// May
  May,
  /// June
  June,
  /// July
  July,
  /// August
  August,
  /// September
  September,
  /// October
  October,
  /// November
  November,
  /// December
  December,
}

impl Month {
  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u8) -> Result<Self, TimeError> {
    Ok(match num {
      1 => Self::January,
      2 => Self::February,
      3 => Self::March,
      4 => Self::April,
      5 => Self::May,
      6 => Self::June,
      7 => Self::July,
      8 => Self::August,
      9 => Self::September,
      10 => Self::October,
      11 => Self::November,
      12 => Self::December,
      _ => return Err(TimeError::InvalidMonth { received: num }),
    })
  }

  /// The number of days given an arbitrary year.
  #[inline]
  pub const fn days(self, year: Year) -> u8 {
    if let Self::February = self {
      if year.is_leap_year() { 29 } else { 28 }
    } else {
      let num = self.num();
      30 | num ^ (num >> 3)
    }
  }

  /// Integer representation
  #[inline]
  pub const fn num(self) -> u8 {
    match self {
      Self::January => 1,
      Self::February => 2,
      Self::March => 3,
      Self::April => 4,
      Self::May => 5,
      Self::June => 6,
      Self::July => 7,
      Self::August => 8,
      Self::September => 9,
      Self::October => 10,
      Self::November => 11,
      Self::December => 12,
    }
  }

  /// String representation
  #[inline]
  pub const fn num_str(self) -> &'static str {
    match self {
      Self::January => "01",
      Self::February => "02",
      Self::March => "03",
      Self::April => "04",
      Self::May => "05",
      Self::June => "06",
      Self::July => "07",
      Self::August => "08",
      Self::September => "09",
      Self::October => "10",
      Self::November => "11",
      Self::December => "12",
    }
  }
}

impl TryFrom<u8> for Month {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
