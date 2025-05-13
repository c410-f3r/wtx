use crate::{
  collection::ArrayVector,
  misc::{FromRadix10 as _, i16_string, u32_string},
  time::{Date, DateTime, Month, Nanosecond, Time, TimeError, Weekday, date_time::DateTimeString},
};

impl DateTime {
  /// Parses a sequence of bytes according to the specified `fmt`.
  ///
  /// # Date Formats
  ///
  /// | Format  | Example  | Description                                                           |
  /// | ------- | -------- | --------------------------------------------------------------------- |
  /// | `%Y`    | `2001`   | Year with four zero-padded digits                                     |
  /// | `%y`    | `01`     | Year with two zero-padded digits                                      |
  /// |         |          |                                                                       |
  /// | `%m`    | `07`     | Month with two zero-padded digits                                     |
  /// | `%b`    | `Jul`    | Abbreviated month name                                                |
  /// |         |          |                                                                       |
  /// | `%d`    | `08`     | Day of month with two zero-padded digits                              |
  /// | `%e`    | ` 8`     | Same as `%d` but space-padded                                         |
  /// |         |          |                                                                       |
  /// | `%a`    | `Sun`    | Abbreviated weekday name                                              |
  /// | `%A`    | `Sunday` | Full weekday name                                                     |
  ///
  /// # Time Formats
  ///
  /// | Format  | Example  | Description                                                           |
  /// | ------- | -------- | --------------------------------------------------------------------- |
  /// | `%H`    | `00`     | Year with two zero-padded digits                                      |
  /// |         |          |                                                                       |
  /// | `%M`    | `59`     | Minute with two zero-padded digits                                    |
  /// |         |          |                                                                       |
  /// | `%S`    | `59`     | Second with two zero-padded digits                                    |
  /// |         |          |                                                                       |
  /// | `%.f`   | `.12345` | Optional number of nanoseconds with a dot prefix                      |
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
  /// | `Z`     | UTC                 |
  #[expect(clippy::too_many_lines, reason = "enum is exhaustive")]
  #[inline]
  pub fn parse(mut data: &[u8], fmt: &[u8]) -> crate::Result<Self> {
    let tokens: ArrayVector<Token, 16> = lexer(fmt)?;
    let mut day_opt = None;
    let mut hour_opt = None;
    let mut minute_opt = None;
    let mut month_opt = None;
    let mut nanos_opt = None;
    let mut second_opt = None;
    let mut weekday_opt = None;
    let mut year_opt = None;
    for token in tokens {
      let rhs = match token {
        Token::AbbreviatedMonthName => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(data, 3)?;
          month_opt = Some(Month::from_short_name(lhs)?);
          rhs
        }
        Token::AbbreviatedWeekdayName => {
          if weekday_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatWeekday.into());
          }
          let (lhs, rhs) = split_at(data, 3)?;
          weekday_opt = Some(Weekday::from_short_name(lhs)?);
          rhs
        }
        Token::Colon => parse_literal(b":", data)?,
        Token::Comma => parse_literal(b",", data)?,
        Token::Dash => parse_literal(b"-", data)?,
        Token::DotNano => {
          let Ok(rest) = parse_literal(b".", data) else {
            continue;
          };
          let mut idx: usize = 0;
          loop {
            let Some(elem) = rest.get(idx) else {
              break;
            };
            if !elem.is_ascii_digit() {
              break;
            }
            idx = idx.wrapping_add(1);
          }
          let (num, rhs) = rest.split_at_checked(idx).unwrap_or_default();
          nanos_opt = Some(u32::from_radix_10(num)?);
          rhs
        }
        Token::FourDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(data, 4)?;
          year_opt = Some(i16::from_radix_10(lhs)?);
          rhs
        }
        Token::FullWeekdayName => {
          if weekday_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatWeekday.into());
          }
          let (weekday, rhs) = Weekday::from_name_relaxed(data)?;
          weekday_opt = Some(weekday);
          rhs
        }
        Token::Gmt => parse_literal(b"GMT", data)?,
        Token::Separator => parse_literal(b"T", data)?,
        Token::Slash => parse_literal(b"/", data)?,
        Token::Space => parse_literal(b" ", data)?,
        Token::TwoDigitDay => {
          if day_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatDay.into());
          }
          let (lhs, rhs) = split_at(data, 2)?;
          day_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitHour => {
          let (lhs, rhs) = split_at(data, 2)?;
          hour_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitMinute => {
          let (lhs, rhs) = split_at(data, 2)?;
          minute_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitMonth => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(data, 2)?;
          month_opt = Some(Month::from_num(u8::from_radix_10(lhs)?)?);
          rhs
        }
        Token::TwoDigitSecond => {
          let (lhs, rhs) = split_at(data, 2)?;
          second_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(data, 2)?;
          let year = i16::from_radix_10(lhs)?;
          if !(0..=99).contains(&year) {
            return Err(TimeError::InvalidParsingData.into());
          }
          year_opt = Some(year.wrapping_add(2000));
          rhs
        }
        Token::TwoSpaceDay => {
          if day_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatDay.into());
          }
          let Some(([a, b], rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          if *a == b' ' {
            day_opt = Some(u8::from_radix_10(&[*b])?);
          } else {
            day_opt = Some(u8::from_radix_10(&[*a, *b])?);
          }
          rhs
        }
        Token::Utc => parse_literal(b"Z", data)?,
      };
      data = rhs;
    }
    let (Some(day), Some(hour), Some(minute), Some(month), Some(second), Some(year)) =
      (day_opt, hour_opt, minute_opt, month_opt, second_opt, year_opt)
    else {
      return Err(TimeError::IncompleteParsingParams.into());
    };
    let nano = if let Some(elem) = nanos_opt { elem.try_into()? } else { Nanosecond::ZERO };
    let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
    let time = Time::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano);
    if let Some(weekday) = weekday_opt {
      if weekday != date.weekday() {
        return Err(TimeError::InvalidParsingWeekday.into());
      }
    }
    Ok(Self::new(date, time))
  }

  /// Creates a string represented based on the given `fmt`.
  #[inline]
  pub fn format(&self, fmt: &[u8]) -> crate::Result<DateTimeString> {
    let tokens: ArrayVector<Token, 16> = lexer(fmt)?;
    let mut string = DateTimeString::new();
    for token in tokens {
      match token {
        Token::AbbreviatedMonthName => {
          string.push_str(self.date.month().short_name())?;
        }
        Token::AbbreviatedWeekdayName => {
          string.push_str(self.date.weekday().short_name())?;
        }
        Token::Colon => {
          string.push(':')?;
        }
        Token::Comma => {
          string.push(',')?;
        }
        Token::Dash => {
          string.push('-')?;
        }
        Token::DotNano => {
          string.push('.')?;
          string.push_str(&u32_string(self.time.nanosecond().num()))?;
        }
        Token::FourDigitYear => {
          let year = i16_string(self.date.year().num());
          let (num, zeros) = if year.len() <= 4 {
            if let [b'-', rest @ ..] = year.as_bytes() {
              string.push('-')?;
              (rest, 5u32.wrapping_sub(year.len()))
            } else {
              (year.as_bytes(), 4u32.wrapping_sub(year.len()))
            }
          } else {
            (year.as_bytes(), 0)
          };
          for _ in 0..zeros {
            string.push('0')?;
          }
          for elem in num {
            string.push((*elem).into())?;
          }
        }
        Token::FullWeekdayName => {
          string.push_str(self.date.weekday().name())?;
        }
        Token::Gmt => {
          string.push_str("GMT")?;
        }
        Token::Separator => {
          string.push('T')?;
        }
        Token::Slash => {
          string.push('/')?;
        }
        Token::Space => {
          string.push(' ')?;
        }
        Token::TwoDigitDay => {
          string.push_str(self.date.day().num_str())?;
        }
        Token::TwoDigitHour => {
          string.push_str(self.time.hour().num_str())?;
        }
        Token::TwoDigitMinute => {
          string.push_str(self.time.minute().num_str())?;
        }
        Token::TwoDigitMonth => {
          string.push_str(self.date.month().num_str())?;
        }
        Token::TwoDigitSecond => {
          string.push_str(self.time.second().num_str())?;
        }
        Token::TwoDigitYear => {
          string.push_str(&i16_string(self.date.year().num().rem_euclid(100)))?;
        }
        Token::TwoSpaceDay => {
          let [a, b] = self.date.day().num_str().as_bytes() else {
            continue;
          };
          if *a == b'0' {
            string.push_str(self.date.day().num_str())?;
          } else {
            string.push((*a).into())?;
            string.push((*b).into())?;
          }
        }
        Token::Utc => {
          string.push('Z')?;
        }
      }
    }
    Ok(string)
  }
}

#[derive(Debug)]
enum Token {
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
  /// `%.f` `123_456_789`
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
  /// Literal `Z`
  Utc,
}

impl TryFrom<[u8; 2]> for Token {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
    Ok(match value {
      [0, b'b'] => Self::AbbreviatedMonthName,
      [0, b'a'] => Self::AbbreviatedWeekdayName,
      [0, b':'] => Self::Colon,
      [0, b','] => Self::Comma,
      [0, b'-'] => Self::Dash,
      [b'.', b'f'] => Self::DotNano,
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
      [0, b'Z'] => Self::Utc,
      _ => return Err(TimeError::UnknownParsingFormat.into()),
    })
  }
}

#[inline]
fn lexer(fmt: &[u8]) -> crate::Result<ArrayVector<Token, 16>> {
  let mut tokens = ArrayVector::new();
  let mut iter = fmt.iter().copied().peekable();
  loop {
    let Some(first) = iter.next() else {
      break;
    };
    match first {
      b'%' => {
        let Some(second) = iter.next() else {
          return Err(TimeError::InvalidParsingFormat.into());
        };
        if second == b'.' {
          let Some(third) = iter.next() else {
            return Err(TimeError::InvalidParsingFormat.into());
          };
          tokens.push([second, third].try_into()?)?;
        } else {
          tokens.push([0, second].try_into()?)?;
        }
      }
      b'G' => {
        let (Some(b'M'), Some(b'T')) = (iter.next(), iter.next()) else {
          return Err(TimeError::InvalidParsingFormat.into());
        };
        tokens.push(Token::Gmt)?;
      }
      _ => {
        tokens.push([0, first].try_into()?)?;
      }
    }
  }
  Ok(tokens)
}

#[inline]
fn parse_literal<'value>(lit: &[u8], value: &'value [u8]) -> crate::Result<&'value [u8]> {
  let Some((lhs, rhs)) = value.split_at_checked(lit.len()) else {
    return Err(TimeError::InvalidParsingData.into());
  };
  if lhs != lit {
    return Err(TimeError::InvalidParsingLiteral.into());
  }
  Ok(rhs)
}

#[inline]
fn split_at(data: &[u8], mid: usize) -> crate::Result<(&[u8], &[u8])> {
  let Some(elem) = data.split_at_checked(mid) else {
    return Err(TimeError::InvalidParsingData.into());
  };
  Ok(elem)
}

#[cfg(test)]
mod tests {
  use crate::time::DateTime;

  static _0_DATA: &[u8] = b"Mon, 12 May 2025 14:30:00 GMT";
  static _0_FMT: &[u8] = b"%a, %d %b %Y %H:%M:%S GMT";

  static _1_DATA: &[u8] = b"Monday, 12-May-25 14:30:00 GMT";
  static _1_FMT: &[u8] = b"%A, %d-%b-%y %H:%M:%S GMT";

  static _2_DATA: &[u8] = b"Mon May 12 14:30:00 2025";
  static _2_FMT: &[u8] = b"%a %b %e %H:%M:%S %Y";

  static _3_DATA: &[u8] = b"Mon, 12-May-2025 14:30:00 GMT";
  static _3_FMT: &[u8] = b"%a, %d-%b-%Y %H:%M:%S GMT";

  static _4_DATA: &[u8] = b"1999-02-03T23:40:20.1234Z";
  static _4_FMT: &[u8] = b"%Y-%m-%dT%H:%M:%S%.fZ";

  #[test]
  fn parse_and_format() {
    assert_eq!(
      DateTime::parse(_0_DATA, _0_FMT).unwrap().format(_0_FMT).unwrap().as_str().as_bytes(),
      _0_DATA
    );
    assert_eq!(
      DateTime::parse(_1_DATA, _1_FMT).unwrap().format(_1_FMT).unwrap().as_str().as_bytes(),
      _1_DATA
    );
    assert_eq!(
      DateTime::parse(_2_DATA, _2_FMT).unwrap().format(_2_FMT).unwrap().as_str().as_bytes(),
      _2_DATA
    );
    assert_eq!(
      DateTime::parse(_3_DATA, _3_FMT).unwrap().format(_3_FMT).unwrap().as_str().as_bytes(),
      _3_DATA
    );
    assert_eq!(
      DateTime::parse(_4_DATA, _4_FMT).unwrap().format(_4_FMT).unwrap().as_str().as_bytes(),
      _4_DATA
    );
  }
}
