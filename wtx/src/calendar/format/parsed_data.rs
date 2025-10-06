use crate::{
  calendar::{
    CalendarError, CalendarToken, Date, DateTime, Hour, Minute, Month, Nanosecond, Time, TimeZone,
    Weekday,
  },
  de::FromRadix10 as _,
};

pub(crate) enum ParsedData<TZ> {
  Time(Time),
  Date(Date),
  DateTime(DateTime<TZ>),
}

impl<TZ> ParsedData<TZ>
where
  TZ: TimeZone,
{
  #[allow(clippy::too_many_lines, reason = "enum is exhaustive")]
  #[inline]
  pub(crate) fn new(
    mut bytes: &[u8],
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<Self> {
    let mut day_opt = None;
    let mut hour_opt = None;
    let mut minute_opt = None;
    let mut month_opt = None;
    let mut nanos_opt = None;
    let mut second_opt = None;
    let mut time_zone_opt = None;
    let mut weekday_opt = None;
    let mut year_opt = None;
    for token in tokens {
      let rhs = match token {
        CalendarToken::AbbreviatedMonthName => {
          if month_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(bytes, 3)?;
          month_opt = Some(Month::from_short_name(lhs)?);
          rhs
        }
        CalendarToken::AbbreviatedWeekdayName => {
          if weekday_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatWeekday.into());
          }
          let (lhs, rhs) = split_at(bytes, 3)?;
          weekday_opt = Some(Weekday::from_short_name(lhs)?);
          rhs
        }
        CalendarToken::Colon => parse_token_literal(b":", bytes)?,
        CalendarToken::Comma => parse_token_literal(b",", bytes)?,
        CalendarToken::Dash => parse_token_literal(b"-", bytes)?,
        CalendarToken::DotNano => {
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
        CalendarToken::FourDigitYear => {
          if year_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(bytes, 4)?;
          year_opt = Some(i16::from_radix_10(lhs)?);
          rhs
        }
        CalendarToken::FullWeekdayName => {
          if weekday_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatWeekday.into());
          }
          let (weekday, rhs) = Weekday::from_name_relaxed(bytes)?;
          weekday_opt = Some(weekday);
          rhs
        }
        CalendarToken::Gmt => parse_token_literal(b"GMT", bytes)?,
        CalendarToken::Separator => parse_token_literal(b"T", bytes)?,
        CalendarToken::Slash => parse_token_literal(b"/", bytes)?,
        CalendarToken::Space => parse_token_literal(b" ", bytes)?,
        CalendarToken::TimeZone => manage_time_zone(
          bytes,
          &mut time_zone_opt,
          |local_time_zone_opt, is_neg, hour, after_hour| {
            const fn change_sign(num: i16, is_neg: bool) -> i16 {
              #[expect(clippy::arithmetic_side_effects, reason = "callers never pass `i16::MAX`")]
              if is_neg { -num } else { num }
            }

            if let Some((minute, after_minute)) = minute(after_hour) {
              *local_time_zone_opt = Some(change_sign(hour.wrapping_add(minute), is_neg));
              Ok(after_minute)
            } else {
              *local_time_zone_opt = Some(change_sign(hour, is_neg));
              Ok(after_hour)
            }
          },
        )?,
        CalendarToken::TwoDigitDay => {
          if day_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatDay.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          day_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        CalendarToken::TwoDigitHour => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          hour_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        CalendarToken::TwoDigitMinute => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          minute_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        CalendarToken::TwoDigitMonth => {
          if month_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatMonth.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          month_opt = Some(Month::from_num(u8::from_radix_10(lhs)?)?);
          rhs
        }
        CalendarToken::TwoDigitSecond => {
          let (lhs, rhs) = split_at(bytes, 2)?;
          second_opt = Some(u8::from_radix_10(lhs)?);
          rhs
        }
        CalendarToken::TwoDigitYear => {
          if year_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatYear.into());
          }
          let (lhs, rhs) = split_at(bytes, 2)?;
          let year = i16::from_radix_10(lhs)?;
          if !(0..=99).contains(&year) {
            return Err(CalendarError::InvalidParsingBytes.into());
          }
          year_opt = Some(year.wrapping_add(2000));
          rhs
        }
        CalendarToken::TwoSpaceDay => {
          if day_opt.is_some() {
            return Err(CalendarError::DuplicatedParsingFormatDay.into());
          }
          let Some(([a, b], rhs)) = bytes.split_at_checked(2) else {
            return Err(CalendarError::InvalidParsingBytes.into());
          };
          if *a == b' ' {
            day_opt = Some(u8::from_radix_10(&[*b])?);
          } else {
            day_opt = Some(u8::from_radix_10(&[*a, *b])?);
          }
          rhs
        }
      };
      bytes = rhs;
    }
    if !bytes.is_empty() {
      return Err(CalendarError::InvalidParsingBytes.into());
    }
    let nano = if let Some(elem) = nanos_opt { elem.try_into()? } else { Nanosecond::ZERO };
    match (year_opt, month_opt, day_opt, hour_opt, minute_opt, second_opt) {
      (None, None, None, Some(hour), Some(minute), Some(second)) => Ok(Self::Time(
        Time::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano),
      )),
      (Some(year), Some(month), Some(day), None, None, None) => {
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, weekday_opt)?;
        Ok(Self::Date(date))
      }
      (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) => {
        let tz_minutes = time_zone_opt.unwrap_or(0);
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, weekday_opt)?;
        Ok(Self::DateTime(DateTime::new(
          date,
          Time::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano),
          TZ::from_minutes(tz_minutes)?,
        )))
      }
      _ => Err(CalendarError::IncompleteParsingParams.into()),
    }
  }
}

fn check_weekday(date: Date, weekday_opt: Option<Weekday>) -> crate::Result<()> {
  if let Some(weekday) = weekday_opt
    && weekday != date.weekday()
  {
    return Err(CalendarError::InvalidParsingWeekday.into());
  }
  Ok(())
}

fn hour(first: u8, bytes: &[u8]) -> crate::Result<(bool, i16, &[u8])> {
  let (is_neg, array, rest) = match (first, bytes) {
    (b'-', [a, b, rest @ ..]) => (true, [*a, *b], rest),
    (b'+', [a, b, rest @ ..]) => (false, [*a, *b], rest),
    _ => {
      return Err(CalendarError::InvalidParsingTimezone.into());
    }
  };
  let hour = Hour::from_num(u8::from_radix_10(&array)?)?;
  Ok((is_neg, i16::from(hour.num()).wrapping_mul(60), rest))
}

fn manage_time_zone<'bytes>(
  bytes: &'bytes [u8],
  time_zone_opt: &mut Option<i16>,
  cb: impl FnOnce(&mut Option<i16>, bool, i16, &'bytes [u8]) -> crate::Result<&'bytes [u8]>,
) -> crate::Result<&'bytes [u8]> {
  if time_zone_opt.is_some() {
    return Err(CalendarError::DuplicatedTimeZone.into());
  }
  let [first, after_first @ ..] = bytes else {
    return Ok(bytes);
  };
  if *first == b'Z' {
    *time_zone_opt = Some(0);
    Ok(after_first)
  } else {
    let (is_neg, hour, after_hour) = hour(*first, after_first)?;
    cb(time_zone_opt, is_neg, hour, after_hour)
  }
}

fn minute(bytes: &[u8]) -> Option<(i16, &[u8])> {
  let [first, after_first @ ..] = bytes else {
    return None;
  };
  let rest = if *first == b':' { after_first } else { bytes };
  let [a, b, after_minute @ ..] = rest else {
    return None;
  };
  let minute = Minute::from_num(u8::from_radix_10(&[*a, *b]).ok()?).ok()?;
  Some((i16::from(minute.num()), after_minute))
}

fn parse_token_literal<'value>(lit: &[u8], value: &'value [u8]) -> crate::Result<&'value [u8]> {
  let Some((lhs, rhs)) = value.split_at_checked(lit.len()) else {
    return Err(CalendarError::InvalidParsingBytes.into());
  };
  if lhs != lit {
    return Err(CalendarError::InvalidParsingLiteral.into());
  }
  Ok(rhs)
}

#[track_caller]
fn split_at(data: &[u8], mid: usize) -> crate::Result<(&[u8], &[u8])> {
  let Some(elem) = data.split_at_checked(mid) else {
    return Err(CalendarError::InvalidParsingBytes.into());
  };
  Ok(elem)
}
