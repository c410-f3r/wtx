use crate::{
  calendar::{CalendarError, TimeZone},
  collection::{ArrayString, ArrayStringU8, IndexedStorageMut as _},
};

/// Universal Time Coordinated (UTC)
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Utc;

impl TimeZone for Utc {
  const IS_LOCAL: bool = false;
  const IS_UTC: bool = true;

  #[inline]
  fn from_minutes(minutes: i16) -> crate::Result<Self> {
    if minutes != 0 {
      return Err(
        CalendarError::InvalidTimezoneSeconds { expected: Some(0), received: minutes }.into(),
      );
    }
    Ok(Self)
  }

  #[inline]
  fn iso_8601(self) -> ArrayStringU8<6> {
    let mut str = ArrayString::new();
    let _rslt = str.push('Z');
    str
  }

  #[inline]
  fn minutes(&self) -> i16 {
    0
  }
}
