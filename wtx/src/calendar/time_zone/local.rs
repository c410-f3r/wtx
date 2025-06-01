use crate::{
  calendar::{CalendarError, TimeZone},
  collection::ArrayString,
};

/// Local time.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Local;

impl TimeZone for Local {
  const IS_LOCAL: bool = true;
  const IS_UTC: bool = false;

  #[inline]
  fn from_minutes(minutes: i16) -> crate::Result<Self> {
    if minutes != 0 {
      return Err(
        CalendarError::InvalidTimezoneSeconds { expected: None, received: minutes }.into(),
      );
    };
    Ok(Self)
  }

  #[inline]
  fn iso_8601(self) -> ArrayString<6> {
    ArrayString::new()
  }

  #[inline]
  fn minutes(&self) -> i16 {
    0
  }
}
