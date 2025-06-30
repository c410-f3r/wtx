use crate::calendar::CalendarError;

/// Semantical unit usually extracted from a stream of bytes
///
/// # Date Formats
///
/// | Format  | Example  | Description                                                                        |
/// | ------- | -------- | ---------------------------------------------------------------------------------- |
/// | `%Y`    | `2001`   | Year with four zero-padded digits                                                  |
/// | `%y`    | `01`     | Year with two zero-padded digits                                                   |
/// |         |          |                                                                                    |
/// | `%m`    | `07`     | Month with two zero-padded digits                                                  |
/// | `%b`    | `Jul`    | Abbreviated month name                                                             |
/// |         |          |                                                                                    |
/// | `%d`    | `08`     | Day of month with two zero-padded digits                                           |
/// | `%e`    | ` 8`     | Same as `%d` but space-padded                                                      |
/// |         |          |                                                                                    |
/// | `%a`    | `Sun`    | Abbreviated weekday name                                                           |
/// | `%A`    | `Sunday` | Full weekday name                                                                  |
/// |         |          |                                                                                    |
/// | `%z?`   | `±03`    | Optional offset from the local time to UTC with or wihtout `Z`, colon and minutes¹ |
///
/// # Time Formats
///
/// | Format  | Example  | Description                                     |
/// | ------- | -------- | ----------------------------------------------- |
/// | `%H`    | `00`     | Year with two zero-padded digits                |
/// |         |          |                                                 |
/// | `%M`    | `59`     | Minute with two zero-padded digits              |
/// |         |          |                                                 |
/// | `%S`    | `59`     | Second with two zero-padded digits              |
/// |         |          |                                                 |
/// | `%f?`   | `.12345` | Optional number of nanosecond with a dot prefix |
///
/// # Literal Formats
///
/// | Format  | Name                |
/// | ------- | ------------------- |
/// | `:`     | Colon               |
/// | `,`     | Comma               |
/// | `-`     | Dash                |
/// | `GMT`   | Greenwich Mean Time |
/// | `/`     | Slash               |
/// | ` `     | Space               |
/// | `T`     | Date/Time separator |
///
/// ¹: Decoding accept many optinal paramenters but encoding will always output ±00:00 or `Z`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CalendarToken {
  /// `%b` (Jul)
  AbbreviatedMonthName,
  /// `%a` (Sun)
  AbbreviatedWeekdayName,
  /// Literal `:`
  Colon,
  /// Literal `,`
  Comma,
  /// Literal `-`
  Dash,
  /// Optional `%f?` `123_456_789`
  DotNano,
  /// `%Y` (2001)
  FourDigitYear,
  /// `%A` (Sunday)
  FullWeekdayName,
  /// Literal `GMT`
  Gmt,
  /// Literal `T`
  Separator,
  /// Literal `/`
  Slash,
  /// Literal ` `
  Space,
  /// Optional `%z?` (`Z`, +03, -0300, +03:30)
  TimeZone,
  /// `%d` (01)
  TwoDigitDay,
  /// `%H` (00)
  TwoDigitHour,
  /// `%M` (00)
  TwoDigitMinute,
  /// `%m` (01)
  TwoDigitMonth,
  /// `%S` (00)
  TwoDigitSecond,
  /// `%y` (25)
  TwoDigitYear,
  /// `%e` ( 1)
  TwoSpaceDay,
}

impl TryFrom<[u8; 2]> for CalendarToken {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
    Ok(match value {
      [0, b'b'] => Self::AbbreviatedMonthName,
      [0, b'a'] => Self::AbbreviatedWeekdayName,
      [0, b':'] => Self::Colon,
      [0, b','] => Self::Comma,
      [0, b'-'] => Self::Dash,
      [b'f', b'?'] => Self::DotNano,
      [0, b'Y'] => Self::FourDigitYear,
      [0, b'A'] => Self::FullWeekdayName,
      [0, b'T'] => Self::Separator,
      [0, b'/'] => Self::Slash,
      [0, b' '] => Self::Space,
      [0, b'd'] => Self::TwoDigitDay,
      [0, b'H'] => Self::TwoDigitHour,
      [0, b'M'] => Self::TwoDigitMinute,
      [0, b'm'] => Self::TwoDigitMonth,
      [0, b'S'] => Self::TwoDigitSecond,
      [0, b'y'] => Self::TwoDigitYear,
      [0, b'e'] => Self::TwoSpaceDay,
      [0, b'Z'] | [b'z', b'?'] => Self::TimeZone,
      _ => return Err(CalendarError::UnknownParsingFormat.into()),
    })
  }
}
