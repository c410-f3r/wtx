use crate::{
  calendar::{
    CalendarError, CalendarToken, DateTime, TimeZone,
    format::{
      parsed_data::ParsedData,
      push::{push_four_digit_year, push_two_space_day},
    },
  },
  collection::{ArrayString, ArrayStringU8},
  de::{i16_string, u32_string},
};

impl<TZ> DateTime<TZ>
where
  TZ: TimeZone,
{
  /// Parses a sequence of bytes according to the specified tokens.
  ///
  /// See [`CalendarToken`] for more information.
  #[inline]
  pub fn parse(
    bytes: &[u8],
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<Self> {
    let ParsedData::DateTime(elem) = ParsedData::new(bytes, tokens)? else {
      return Err(CalendarError::InvalidParsingDateTime.into());
    };
    Ok(elem)
  }

  /// Creates a string representation based on the given `tokens`.
  ///
  /// A string of 32 bytes is usually more than enough for most representations.
  ///
  /// See [`CalendarToken`] for more information.
  #[inline]
  pub fn to_string<const N: usize>(
    &self,
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<ArrayStringU8<N>> {
    let mut string = ArrayString::new();
    for token in tokens {
      match token {
        CalendarToken::AbbreviatedMonthName => {
          string.push_str(self.date.month().short_name())?;
        }
        CalendarToken::AbbreviatedWeekdayName => {
          string.push_str(self.date.weekday().short_name())?;
        }
        CalendarToken::Colon => {
          string.push(':')?;
        }
        CalendarToken::Comma => {
          string.push(',')?;
        }
        CalendarToken::Dash => {
          string.push('-')?;
        }
        CalendarToken::DotNano => {
          string.push('.')?;
          string.push_str(&u32_string(self.time.nanosecond().num()))?;
        }
        CalendarToken::FourDigitYear => {
          push_four_digit_year(self.date, &mut string)?;
        }
        CalendarToken::FullWeekdayName => {
          string.push_str(self.date.weekday().name())?;
        }
        CalendarToken::Gmt => {
          string.push_str("GMT")?;
        }
        CalendarToken::Separator => {
          string.push('T')?;
        }
        CalendarToken::Slash => {
          string.push('/')?;
        }
        CalendarToken::Space => {
          string.push(' ')?;
        }
        CalendarToken::TimeZone => {
          string.push_str(&self.tz.iso_8601())?;
        }
        CalendarToken::TwoDigitDay => {
          string.push_str(self.date.day().num_str())?;
        }
        CalendarToken::TwoDigitHour => {
          string.push_str(self.time.hour().num_str())?;
        }
        CalendarToken::TwoDigitMinute => {
          string.push_str(self.time.minute().num_str())?;
        }
        CalendarToken::TwoDigitMonth => {
          string.push_str(self.date.month().num_str())?;
        }
        CalendarToken::TwoDigitSecond => {
          string.push_str(self.time.second().num_str())?;
        }
        CalendarToken::TwoDigitYear => {
          string.push_str(&i16_string(self.date.year().num().rem_euclid(100)))?;
        }
        CalendarToken::TwoSpaceDay => {
          push_two_space_day(self.date, &mut string)?;
        }
      }
    }
    Ok(string)
  }
}

#[cfg(test)]
mod tests {
  use crate::calendar::{DateTime, DynTz, Local, Utc, format::parse_bytes_into_tokens};

  static _0_DATA: &[u8] = b"Mon, 12 May 2025 14:30:00 GMT";
  static _0_FMT: &[u8] = b"%a, %d %b %Y %H:%M:%S GMT";

  static _1_DATA: &[u8] = b"Monday, 12-May-25 14:30:00 GMT";
  static _1_FMT: &[u8] = b"%A, %d-%b-%y %H:%M:%S GMT";

  static _2_DATA: &[u8] = b"Mon May 12 14:30:00 2025";
  static _2_FMT: &[u8] = b"%a %b %e %H:%M:%S %Y";

  static _3_DATA: &[u8] = b"Mon, 12-May-2025 14:30:00 GMT";
  static _3_FMT: &[u8] = b"%a, %d-%b-%Y %H:%M:%S GMT";

  static _4_DATA: &[u8] = b"1999-02-03T23:40:20.1234Z";
  static _4_FMT: &[u8] = b"%Y-%m-%dT%H:%M:%S%f?Z";

  static _5_DATA0: &[u8] = b"2000-02-01T00:40:20.1234";
  static _5_DATA1: &[u8] = b"2000-02-01T00:40:20.1234Z";
  static _5_DATA2: &[u8] = b"2000-02-01T00:40:20.1234+03";
  static _5_DATA3: &[u8] = b"2000-02-01T00:40:20.1234+0300";
  static _5_DATA4: &[u8] = b"2000-02-01T00:40:20.1234+03:00";
  static _5_DATA5: &[u8] = b"2000-02-01T00:40:20.1234+03:30";
  static _5_DATA6: &[u8] = b"2000-02-01T00:40:20.1234-03";
  static _5_DATA7: &[u8] = b"2000-02-01T00:40:20.1234-0300";
  static _5_DATA8: &[u8] = b"2000-02-01T00:40:20.1234-03:00";
  static _5_DATA9: &[u8] = b"2000-02-01T00:40:20.1234-03:30";
  static _5_FMT: &[u8] = b"%Y-%m-%dT%H:%M:%S%f?%z?";

  #[test]
  fn _0() {
    let _0_tokens = parse_bytes_into_tokens(_0_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Utc>::parse(_0_DATA, _0_tokens.clone())
        .unwrap()
        .to_string::<38>(_0_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _0_DATA
    );
  }

  #[test]
  fn _1() {
    let _1_tokens = parse_bytes_into_tokens(_1_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Utc>::parse(_1_DATA, _1_tokens.clone())
        .unwrap()
        .to_string::<38>(_1_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _1_DATA
    );
  }

  #[test]
  fn _2() {
    let _2_tokens = parse_bytes_into_tokens(_2_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Utc>::parse(_2_DATA, _2_tokens.clone())
        .unwrap()
        .to_string::<38>(_2_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _2_DATA
    );
  }

  #[test]
  fn _3() {
    let _3_tokens = parse_bytes_into_tokens(_3_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Utc>::parse(_3_DATA, _3_tokens.clone())
        .unwrap()
        .to_string::<38>(_3_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _3_DATA
    );
  }

  #[test]
  fn _4() {
    let _4_tokens = parse_bytes_into_tokens(_4_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Utc>::parse(_4_DATA, _4_tokens.clone())
        .unwrap()
        .to_string::<38>(_4_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _4_DATA
    );
  }

  #[test]
  fn _5() {
    let _5_tokens = parse_bytes_into_tokens(_5_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::<Local>::parse(_5_DATA0, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA0
    );
    assert_eq!(
      DateTime::<Utc>::parse(_5_DATA1, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA1
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA2, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA4
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA3, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA4
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA4, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA4
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA5, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA5
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA6, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA8
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA7, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA8
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA8, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens.clone())
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA8
    );
    assert_eq!(
      DateTime::<DynTz>::parse(_5_DATA9, _5_tokens.clone())
        .unwrap()
        .to_string::<38>(_5_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _5_DATA9
    );
  }
}
