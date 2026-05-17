use crate::{
  calendar::{
    CalendarError, CalendarToken, Date, DateTime, Hour, Month, Nanosecond, Sixty, Time, TimeZone,
    Weekday,
  },
  codec::FromRadix10 as _,
};

const NANO_MULTIPLIERS: &[u32; 10] =
  &[1_000_000_000, 100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1_000, 100, 10, 1];

pub(crate) enum ParsedData<TZ> {
  Time(Time),
  Date(Date),
  DateTime(DateTime<TZ>),
}

impl<TZ> ParsedData<TZ>
where
  TZ: TimeZone,
{
  #[inline]
  pub(crate) fn new(
    mut bytes: &[u8],
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<Self> {
    let mut params = Params::default();
    for token in tokens {
      if let Some(elem) = process_token(bytes, &mut params, token)? {
        bytes = elem;
      }
    }
    if !bytes.is_empty() {
      return Err(CalendarError::InvalidParsingBytes.into());
    }
    let nano = if let Some(elem) = params.nanos { elem.try_into()? } else { Nanosecond::ZERO };
    match (params.year, params.month, params.day, params.hour, params.minute, params.second) {
      (None, None, None, Some(hour), Some(minute), Some(second)) => Ok(Self::Time(
        Time::from_hms_ns(hour.try_into()?, minute.try_into()?, second.try_into()?, nano),
      )),
      (Some(year), Some(month), Some(day), None, None, None) => {
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, params.weekday)?;
        Ok(Self::Date(date))
      }
      (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) => {
        let tz_minutes = params.time_zone.unwrap_or(0);
        let date = Date::from_ymd(year.try_into()?, month, day.try_into()?)?;
        check_weekday(date, params.weekday)?;
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
    (b'-', [b0, b1, rest @ ..]) => (true, [*b0, *b1], rest),
    (b'+', [b0, b1, rest @ ..]) => (false, [*b0, *b1], rest),
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
  let [b0, b1, after_minute @ ..] = rest else {
    return None;
  };
  let minute = Sixty::from_num(u8::from_radix_10(&[*b0, *b1]).ok()?).ok()?;
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

#[expect(clippy::too_many_lines, reason = "enum is exhaustive")]
fn process_token<'bytes>(
  bytes: &'bytes [u8],
  params: &mut Params,
  token: CalendarToken,
) -> crate::Result<Option<&'bytes [u8]>> {
  let rslt = match token {
    CalendarToken::AbbreviatedMonthName => {
      if params.month.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatMonth.into());
      }
      let (lhs, rhs) = split_at(bytes, 3)?;
      params.month = Some(Month::from_short_name(lhs)?);
      rhs
    }
    CalendarToken::AbbreviatedWeekdayName => {
      if params.weekday.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatWeekday.into());
      }
      let (lhs, rhs) = split_at(bytes, 3)?;
      params.weekday = Some(Weekday::from_short_name(lhs)?);
      rhs
    }
    CalendarToken::Colon => parse_token_literal(b":", bytes)?,
    CalendarToken::Comma => parse_token_literal(b",", bytes)?,
    CalendarToken::Dash => parse_token_literal(b"-", bytes)?,
    CalendarToken::Dot => parse_token_literal(b".", bytes)?,
    CalendarToken::DotNano => {
      let Ok(rest) = parse_token_literal(b".", bytes) else {
        return Ok(None);
      };
      let mut idx: usize = 0;
      while let Some(elem) = rest.get(idx) {
        if !elem.is_ascii_digit() {
          break;
        }
        idx = idx.wrapping_add(1);
      }
      let (num, rhs) = rest.split_at_checked(idx).unwrap_or_default();
      let take_len = idx.min(9);
      let (nano_bytes, _) = num.split_at_checked(take_len).unwrap_or_default();
      let value = u32::from_radix_10(nano_bytes)?;
      let multiplier = NANO_MULTIPLIERS.get(take_len).copied().unwrap_or_default();
      params.nanos = Some(value.wrapping_mul(multiplier));
      rhs
    }
    CalendarToken::FourDigitYear => {
      if params.year.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatYear.into());
      }
      let idx = match bytes {
        [b'-', b0, b1, b2, b3, b4, rest @ ..] if b4.is_ascii_digit() => 6,
        [b0, b1, b2, b3, b4, rest @ ..] if b4.is_ascii_digit() => 5,
        _ => 4,
      };
      let (lhs, rhs) = split_at(bytes, idx)?;
      params.year = Some(i16::from_radix_10(lhs)?);
      rhs
    }
    CalendarToken::FullWeekdayName => {
      if params.weekday.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatWeekday.into());
      }
      let (weekday, rhs) = Weekday::from_name_relaxed(bytes)?;
      params.weekday = Some(weekday);
      rhs
    }
    CalendarToken::Gmt => parse_token_literal(b"GMT", bytes)?,
    CalendarToken::Separator => parse_token_literal(b"T", bytes)?,
    CalendarToken::Slash => parse_token_literal(b"/", bytes)?,
    CalendarToken::Space => parse_token_literal(b" ", bytes)?,
    CalendarToken::TimeZone => manage_time_zone(
      bytes,
      &mut params.time_zone,
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
      if params.day.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatDay.into());
      }
      let (lhs, rhs) = split_at(bytes, 2)?;
      params.day = Some(u8::from_radix_10(lhs)?);
      rhs
    }
    CalendarToken::TwoDigitHour => {
      let (lhs, rhs) = split_at(bytes, 2)?;
      params.hour = Some(u8::from_radix_10(lhs)?);
      rhs
    }
    CalendarToken::TwoDigitMinute => {
      let (lhs, rhs) = split_at(bytes, 2)?;
      params.minute = Some(u8::from_radix_10(lhs)?);
      rhs
    }
    CalendarToken::TwoDigitMonth => {
      if params.month.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatMonth.into());
      }
      let (lhs, rhs) = split_at(bytes, 2)?;
      params.month = Some(Month::from_num(u8::from_radix_10(lhs)?)?);
      rhs
    }
    CalendarToken::TwoDigitSecond => {
      let (lhs, rhs) = split_at(bytes, 2)?;
      params.second = Some(u8::from_radix_10(lhs)?);
      rhs
    }
    CalendarToken::TwoDigitYear => {
      if params.year.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatYear.into());
      }
      let (lhs, rhs) = split_at(bytes, 2)?;
      let year = i16::from_radix_10(lhs)?;
      if !(0..=99).contains(&year) {
        return Err(CalendarError::InvalidParsingBytes.into());
      }
      params.year = Some(year.wrapping_add(2000));
      rhs
    }
    CalendarToken::TwoSpaceDay => {
      if params.day.is_some() {
        return Err(CalendarError::DuplicatedParsingFormatDay.into());
      }
      let Some(([b0, b1], rhs)) = bytes.split_at_checked(2) else {
        return Err(CalendarError::InvalidParsingBytes.into());
      };
      if *b0 == b' ' {
        params.day = Some(u8::from_radix_10(&[*b1])?);
      } else {
        params.day = Some(u8::from_radix_10(&[*b0, *b1])?);
      }
      rhs
    }
  };
  Ok(Some(rslt))
}

#[track_caller]
fn split_at(data: &[u8], mid: usize) -> crate::Result<(&[u8], &[u8])> {
  let Some(elem) = data.split_at_checked(mid) else {
    return Err(CalendarError::InvalidParsingBytes.into());
  };
  Ok(elem)
}

#[derive(Debug, Default)]
struct Params {
  day: Option<u8>,
  hour: Option<u8>,
  minute: Option<u8>,
  month: Option<Month>,
  nanos: Option<u32>,
  second: Option<u8>,
  time_zone: Option<i16>,
  weekday: Option<Weekday>,
  year: Option<i16>,
}
