use crate::{
  collection::ArrayVector,
  misc::{FromRadix10, i16_string},
  time::{Date, DateTime, Month, Time, TimeError, Weekday, date_time::DateTimeString},
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
  #[inline]
  pub fn parse(mut data: &[u8], fmt: &[u8]) -> crate::Result<Self> {
    let tokens: ArrayVector<Token, 16> = lexer(fmt)?;
    let mut day_opt = None;
    let mut hour_opt = None;
    let mut minute_opt = None;
    let mut month_opt = None;
    let mut second_opt = None;
    let mut weekday_opt = None;
    let mut year_opt = None;
    for token in tokens {
      let rhs = match token {
        Token::AbbreviatedMonthName => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(3) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          month_opt = Some(Month::from_short_name(lhs)?);
          rhs
        }
        Token::AbbreviatedWeekdayName => {
          if weekday_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatWeekday.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(3) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          weekday_opt = Some(Weekday::from_short_name(lhs)?);
          rhs
        }
        Token::Colon => parse_literal(b":", &mut data)?,
        Token::Comma => parse_literal(b",", &mut data)?,
        Token::Dash => parse_literal(b"-", &mut data)?,
        Token::FourDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(4) else {
            return Err(TimeError::InvalidParsingData.into());
          };
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
        Token::Gmt => parse_literal(b"GMT", &mut data)?,
        Token::Slash => parse_literal(b"/", &mut data)?,
        Token::Space => parse_literal(b" ", &mut data)?,
        Token::TwoDigitDay => {
          if day_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatDay.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          day_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitHour => {
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          hour_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitMinute => {
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          minute_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitMonth => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          month_opt = Some(Month::from_num(u8::from_radix_10(lhs)?)?);
          rhs
        }
        Token::TwoDigitSecond => {
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
          second_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        Token::TwoDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let Some((lhs, rhs)) = data.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingData.into());
          };
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
      };
      data = rhs;
    }
    let (Some(day), Some(hour), Some(minute), Some(month), Some(second), Some(year)) =
      (day_opt, hour_opt, minute_opt, month_opt, second_opt, year_opt)
    else {
      return Err(TimeError::IncompleteParsingParams.into());
    };
    let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
    let time = Time::from_hms(hour.try_into()?, minute.try_into()?, second.try_into()?);
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
          let year = i16_string(self.date.year().num().rem_euclid(100));
          string.push_str(year.as_str())?;
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
  /// `%Y` (2001)
  FourDigitYear,
  /// `%A` (Sunday)
  FullWeekdayName,
  /// Literal `GMT`
  Gmt,
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
}

impl TryFrom<u8> for Token {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Ok(match value {
      b'b' => Self::AbbreviatedMonthName,
      b'a' => Self::AbbreviatedWeekdayName,
      b':' => Self::Colon,
      b',' => Self::Comma,
      b'-' => Self::Dash,
      b'Y' => Self::FourDigitYear,
      b'A' => Self::FullWeekdayName,
      b'/' => Self::Slash,
      b' ' => Self::Space,
      b'd' => Self::TwoDigitDay,
      b'H' => Self::TwoDigitHour,
      b'M' => Self::TwoDigitMinute,
      b'm' => Self::TwoDigitMonth,
      b'S' => Self::TwoDigitSecond,
      b'y' => Self::TwoDigitYear,
      b'e' => Self::TwoSpaceDay,
      _ => return Err(TimeError::UnknownParsingFormat.into()),
    })
  }
}

#[inline]
fn lexer(fmt: &[u8]) -> crate::Result<ArrayVector<Token, 16>> {
  let mut tokens = ArrayVector::new();
  let mut iter = fmt.iter().copied();
  loop {
    let Some(first) = iter.next() else {
      break;
    };
    match first {
      b'%' => {
        let Some(second) = iter.next() else {
          return Err(TimeError::InvalidParsingFormat.into());
        };
        tokens.push(second.try_into()?)?;
      }
      b'G' => {
        let (Some(b'M'), Some(b'T')) = (iter.next(), iter.next()) else {
          return Err(TimeError::InvalidParsingFormat.into());
        };
        tokens.push(Token::Gmt)?;
      }
      _ => {
        tokens.push(first.try_into()?)?;
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
  }
}
