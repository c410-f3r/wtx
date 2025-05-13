use crate::{
  misc::FromRadix10 as _,
  time::{ClockTime, Date, DateTime, Month, Nanosecond, TimeError, TimeToken, Weekday},
};

pub(crate) enum ParsedData {
  Time(ClockTime),
  Date(Date),
  DateTime(DateTime),
}

impl ParsedData {
  #[allow(clippy::too_many_lines, reason = "enum is exhaustive")]
  #[inline]
  pub(crate) fn new(
    mut bytes: &[u8],
    tokens: impl IntoIterator<Item = TimeToken>,
  ) -> crate::Result<Self> {
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
        TimeToken::AbbreviatedMonthName => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(bytes, 3)?;
          month_opt = Some(Month::from_short_name(lhs)?);
          rhs
        }
        TimeToken::AbbreviatedWeekdayName => {
          if weekday_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatWeekday.into());
          }
          let (lhs, rhs) = split_at(bytes, 3)?;
          weekday_opt = Some(Weekday::from_short_name(lhs)?);
          rhs
        }
        TimeToken::Colon => parse_token_literal(b":", bytes)?,
        TimeToken::Comma => parse_token_literal(b",", bytes)?,
        TimeToken::Dash => parse_token_literal(b"-", bytes)?,
        TimeToken::DotNano => {
          let Ok(rest) = parse_token_literal(b".", bytes) else {
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
        TimeToken::FourDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(bytes, 4)?;
          year_opt = Some(i16::from_radix_10(lhs)?);
          rhs
        }
        TimeToken::FullWeekdayName => {
          if weekday_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatWeekday.into());
          }
          let (weekday, rhs) = Weekday::from_name_relaxed(bytes)?;
          weekday_opt = Some(weekday);
          rhs
        }
        TimeToken::Gmt => parse_token_literal(b"GMT", bytes)?,
        TimeToken::Separator => parse_token_literal(b"T", bytes)?,
        TimeToken::Slash => parse_token_literal(b"/", bytes)?,
        TimeToken::Space => parse_token_literal(b" ", bytes)?,
        TimeToken::TwoDigitDay => {
          if day_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatDay.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          day_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        TimeToken::TwoDigitHour => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          hour_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        TimeToken::TwoDigitMinute => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          minute_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        TimeToken::TwoDigitMonth => {
          if month_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          month_opt = Some(Month::from_num(u8::from_radix_10(lhs)?)?);
          rhs
        }
        TimeToken::TwoDigitSecond => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          second_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        TimeToken::TwoDigitYear => {
          if year_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          let year = i16::from_radix_10(lhs)?;
          if !(0..=99).contains(&year) {
            return Err(TimeError::InvalidParsingBytes.into());
          }
          year_opt = Some(year.wrapping_add(2000));
          rhs
        }
        TimeToken::TwoSpaceDay => {
          if day_opt.is_some() {
            return Err(TimeError::DuplicatedParsingFormatDay.into());
          }
          let Some(([a, b], rhs)) = bytes.split_at_checked(2) else {
            return Err(TimeError::InvalidParsingBytes.into());
          };
          if *a == b' ' {
            day_opt = Some(u8::from_radix_10(&[*b])?);
          } else {
            day_opt = Some(u8::from_radix_10(&[*a, *b])?);
          }
          rhs
        }
        TimeToken::Utc => parse_token_literal(b"Z", bytes)?,
      };
      bytes = rhs;
    }
    let nano = if let Some(elem) = nanos_opt { elem.try_into()? } else { Nanosecond::ZERO };
    match (year_opt, month_opt, day_opt, hour_opt, minute_opt, second_opt) {
      (None, None, None, Some(hour), Some(minute), Some(second)) => Ok(Self::Time(
        ClockTime::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano),
      )),
      (Some(year), Some(month), Some(day), None, None, None) => {
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, weekday_opt)?;
        Ok(Self::Date(date))
      }
      (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) => {
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, weekday_opt)?;
        Ok(Self::DateTime(DateTime::new(
          date,
          ClockTime::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano),
        )))
      }
      _ => Err(TimeError::IncompleteParsingParams.into()),
    }
  }
}

#[inline]
fn check_weekday(date: Date, weekday_opt: Option<Weekday>) -> crate::Result<()> {
  if let Some(weekday) = weekday_opt {
    if weekday != date.weekday() {
      return Err(TimeError::InvalidParsingWeekday.into());
    }
  }
  Ok(())
}

#[inline]
fn parse_token_literal<'value>(lit: &[u8], value: &'value [u8]) -> crate::Result<&'value [u8]> {
  let Some((lhs, rhs)) = value.split_at_checked(lit.len()) else {
    return Err(TimeError::InvalidParsingBytes.into());
  };
  if lhs != lit {
    return Err(TimeError::InvalidParsingLiteral.into());
  }
  Ok(rhs)
}

#[inline]
fn split_at(data: &[u8], mid: usize) -> crate::Result<(&[u8], &[u8])> {
  let Some(elem) = data.split_at_checked(mid) else {
    return Err(TimeError::InvalidParsingBytes.into());
  };
  Ok(elem)
}
