/// Time error
#[derive(Debug)]
pub enum TimeError {
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
    received: u8,
  },
  /// A second can only have up to `999_999_999` nanoseconds
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
  /// Underlying time structure couldn't hold the value generated during an arithmetic operation.
  InvalidTimeArithmetic,
  /// A year be must between `-32767` and `32766`.
  InvalidYear {
    /// Invalid received year
    received: i16,
  },
}
