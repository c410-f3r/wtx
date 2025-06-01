/// Calendar error
#[derive(Debug)]
pub enum CalendarError {
  // Generic
  //
  /// Underlying time structure couldn't hold the value generated during an arithmetic operation.
  ArithmeticOverflow,
  /// Days from CE must be within the `-11967900` ~ `11967535` range
  InvalidCeDays {
    /// Invalid received number
    received: i32,
  },
  /// A month can only have up to 31 days
  InvalidMonthDay {
    /// Invalid received number
    received: u8,
  },
  /// A year can only have up to 366 days
  InvalidDayOfTheYear {
    /// Invalid received number
    received: u16,
  },
  /// Received 366 days in a non-leap year. Non-leap years must have 365 days.
  InvalidDayOfTheYearInNonLeapYear,
  /// The hardware returned an incorrect time value
  InvalidHardwareTime,
  /// A day can only have up to 24 hours
  InvalidHour {
    /// Invalid received number
    received: u8,
  },
  /// A second can only have up to `999_999` microsecond
  InvalidMicrosecond {
    /// Invalid received number
    received: u32,
  },
  /// A second can only have up to `999` milliseconds
  InvalidMillisecond {
    /// Invalid received number
    received: u16,
  },
  /// A hour can only have up to 60 hours
  InvalidMinute {
    /// Invalid received number
    received: u8,
  },
  /// A hour can only have up to 60 hours
  InvalidMonth {
    /// Invalid received number
    received: Option<u8>,
  },
  /// A second can only have up to `999_999_999` nanosecond
  InvalidNanosecond {
    /// Invalid received number
    received: u32,
  },
  /// `Instant` doesn't have a time provider.
  InstantNeedsBackend,
  /// A minute can only have up to 60 seconds
  InvalidSecond {
    /// Invalid received number
    received: u8,
  },
  /// A timestamp in this project can only go up to `32768-12-31`.
  InvalidTimestamp,
  /// Time zone couldn't be constructed with the given seconds
  InvalidTimezoneSeconds {
    /// Expected number of seconds
    expected: Option<i16>,
    /// Invalid received number
    received: i16,
  },
  /// A weekday must be, for example, "Mon" or "Monday"
  InvalidWeekday,
  /// A year be must between `-32767` and `32766`.
  InvalidYear {
    /// Invalid received year
    received: i16,
  },

  // Parsing
  //
  /// Format contains more than one day
  DuplicatedParsingFormatDay,
  /// Format contains more than one month
  DuplicatedParsingFormatMonth,
  /// Format contains more than one weekday
  DuplicatedParsingFormatWeekday,
  /// Format contains more than one year
  DuplicatedParsingFormatYear,
  /// Format contains more than one time zone
  DuplicatedTimeZone,
  /// Missing date or time parameters
  IncompleteParsingParams,
  /// Provided data does not match provided format
  InvalidParsingBytes,
  /// Provided data can not represent a single clock time
  InvalidParsingClockTime,
  /// Provided data can not represent a single date
  InvalidParsingDate,
  /// Provided data can not represent a single datetime
  InvalidParsingDateTime,
  /// Provided format contains invalid syntax
  InvalidParsingFormat,
  /// A literal from the provided format does not match in the provided data
  InvalidParsingLiteral,
  /// Provided data can not represent a timezone
  InvalidParsingTimezone,
  /// The provided weekday is wrong.
  InvalidParsingWeekday,
  /// Provided format contains unknown characters
  UnknownParsingFormat,
}
