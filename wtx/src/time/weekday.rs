use crate::time::TimeError;

/// The day of week.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Weekday {
  /// Monday.
  Monday,
  /// Tuesday.
  Tuesday,
  /// Wednesday.
  Wednesday,
  /// Thursday.
  Thursday,
  /// Friday.
  Friday,
  /// Saturday.
  Saturday,
  /// Sunday.
  Sunday,
}

impl Weekday {
  /// Creates a new instance from a valid `name` like `Monday` or `Sunday`.
  #[inline]
  pub const fn from_name(name: &[u8]) -> Result<Self, TimeError> {
    let (this, rest) = match Self::from_name_relaxed(name) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    if rest.len() > 0 {
      return Err(TimeError::InvalidWeekday);
    }
    Ok(this)
  }

  /// Creates a new instance from a valid short `name` like `Mon` or `Sun`.
  #[inline]
  pub const fn from_short_name(name: &[u8]) -> Result<Self, TimeError> {
    Ok(match name {
      b"Mon" => Self::Monday,
      b"Tue" => Self::Tuesday,
      b"Wed" => Self::Wednesday,
      b"Thu" => Self::Thursday,
      b"Fri" => Self::Friday,
      b"Sat" => Self::Saturday,
      b"Sun" => Self::Sunday,
      _ => return Err(TimeError::InvalidWeekday),
    })
  }

  /// Full name like `Monday` or `Sunday`
  #[inline]
  pub const fn name(&self) -> &'static str {
    match self {
      Self::Monday => "Monday",
      Self::Tuesday => "Tuesday",
      Self::Wednesday => "Wednesday",
      Self::Thursday => "Thursday",
      Self::Friday => "Friday",
      Self::Saturday => "Saturday",
      Self::Sunday => "Sunday",
    }
  }

  /// Short name like `Mon` or `Sun`
  #[inline]
  pub const fn short_name(&self) -> &'static str {
    match self {
      Self::Monday => "Mon",
      Self::Tuesday => "Tue",
      Self::Wednesday => "Wed",
      Self::Thursday => "Thu",
      Self::Friday => "Fri",
      Self::Saturday => "Sat",
      Self::Sunday => "Sun",
    }
  }

  #[inline]
  pub(crate) const fn from_name_relaxed(name: &[u8]) -> Result<(Self, &[u8]), TimeError> {
    Ok(match name {
      [b'M', b'o', b'n', b'd', b'a', b'y', rest @ ..] => (Self::Monday, rest),
      [b'T', b'u', b'e', b's', b'd', b'a', b'y', rest @ ..] => (Self::Tuesday, rest),
      [b'W', b'e', b'd', b'n', b'e', b's', b'd', b'a', b'y', rest @ ..] => (Self::Wednesday, rest),
      [b'T', b'h', b'u', b'r', b's', b'd', b'a', b'y', rest @ ..] => (Self::Thursday, rest),
      [b'F', b'r', b'i', b'd', b'a', b'y', rest @ ..] => (Self::Friday, rest),
      [b'S', b'a', b't', b'u', b'r', b'd', b'a', b'y', rest @ ..] => (Self::Saturday, rest),
      [b'S', b'u', b'n', b'd', b'a', b'y', rest @ ..] => (Self::Sunday, rest),
      _ => return Err(TimeError::InvalidWeekday),
    })
  }
}
