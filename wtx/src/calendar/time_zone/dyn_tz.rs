use crate::{
  calendar::{CalendarError, Hour, MINUTES_PER_HOUR, Minute, TimeZone},
  collection::ArrayString,
};

/// Dynamic Time Zone. From -23:59 to +23:59.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DynTz(i16);

impl DynTz {
  /// Constant version of [`TimeZone::from_minutes`].
  pub const fn new(minutes: i16) -> Result<Self, CalendarError> {
    let -1439..=1439 = minutes else {
      return Err(CalendarError::InvalidTimezoneSeconds { expected: None, received: minutes });
    };
    Ok(Self(minutes))
  }
}

impl TimeZone for DynTz {
  const IS_LOCAL: bool = false;
  const IS_UTC: bool = false;

  #[inline]
  fn from_minutes(minutes: i16) -> crate::Result<Self> {
    Ok(Self::new(minutes)?)
  }

  #[inline]
  fn iso_8601(self) -> ArrayString<6> {
    let mph = i16::from(MINUTES_PER_HOUR);
    // SAFETY: The number of minutes is within the -1439..=1439 range
    let hour = unsafe {
      let elem = (self.0.abs() / mph).try_into().unwrap_or_default();
      Hour::from_num(elem).unwrap_unchecked()
    };
    // SAFETY: Module 60 guarantees bounds
    let minute = unsafe {
      let elem = (self.0.abs() % mph).abs().try_into().unwrap_or_default();
      Minute::from_num(elem).unwrap_unchecked()
    };
    let mut str = ArrayString::new();
    let _rslt = str.push(if self.0 < 0 { '-' } else { '+' });
    let _rslt = str.push_str(hour.num_str());
    let _rslt = str.push(':');
    let _rslt = str.push_str(minute.num_str());
    str
  }

  #[inline]
  fn minutes(&self) -> i16 {
    self.0
  }
}
