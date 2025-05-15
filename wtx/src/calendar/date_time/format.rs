use crate::{
  calendar::{
    CalendarError, DateTime, TimeToken,
    format::{
      parsed_data::ParsedData,
      push::{push_four_digit_year, push_two_space_day},
    },
  },
  collection::ArrayString,
  misc::{i16_string, u32_string},
};

impl DateTime {
  /// Parses a sequence of bytes according to the specified tokens.
  ///
  /// See [`TimeToken`] for more information.
  #[inline]
  pub fn parse(bytes: &[u8], tokens: impl IntoIterator<Item = TimeToken>) -> crate::Result<Self> {
    let ParsedData::DateTime(elem) = ParsedData::new(bytes, tokens)? else {
      return Err(CalendarError::InvalidParsingDateTime.into());
    };
    Ok(elem)
  }

  /// Creates a string representation based on the given `tokens`.
  ///
  /// A string of 32 bytes is usually more than enough for most representations.
  ///
  /// See [`TimeToken`] for more information.
  #[inline]
  pub fn to_string<const N: usize>(
    &self,
    tokens: impl IntoIterator<Item = TimeToken>,
  ) -> crate::Result<ArrayString<N>> {
    let mut string = ArrayString::new();
    for token in tokens {
      match token {
        TimeToken::AbbreviatedMonthName => {
          string.push_str(self.date.month().short_name())?;
        }
        TimeToken::AbbreviatedWeekdayName => {
          string.push_str(self.date.weekday().short_name())?;
        }
        TimeToken::Colon => {
          string.push(':')?;
        }
        TimeToken::Comma => {
          string.push(',')?;
        }
        TimeToken::Dash => {
          string.push('-')?;
        }
        TimeToken::DotNano => {
          string.push('.')?;
          string.push_str(&u32_string(self.time.nanosecond().num()))?;
        }
        TimeToken::FourDigitYear => {
          push_four_digit_year(self.date, &mut string)?;
        }
        TimeToken::FullWeekdayName => {
          string.push_str(self.date.weekday().name())?;
        }
        TimeToken::Gmt => {
          string.push_str("GMT")?;
        }
        TimeToken::Separator => {
          string.push('T')?;
        }
        TimeToken::Slash => {
          string.push('/')?;
        }
        TimeToken::Space => {
          string.push(' ')?;
        }
        TimeToken::TwoDigitDay => {
          string.push_str(self.date.day().num_str())?;
        }
        TimeToken::TwoDigitHour => {
          string.push_str(self.time.hour().num_str())?;
        }
        TimeToken::TwoDigitMinute => {
          string.push_str(self.time.minute().num_str())?;
        }
        TimeToken::TwoDigitMonth => {
          string.push_str(self.date.month().num_str())?;
        }
        TimeToken::TwoDigitSecond => {
          string.push_str(self.time.second().num_str())?;
        }
        TimeToken::TwoDigitYear => {
          string.push_str(&i16_string(self.date.year().num().rem_euclid(100)))?;
        }
        TimeToken::TwoSpaceDay => {
          push_two_space_day(self.date, &mut string)?;
        }
        TimeToken::Utc => {
          string.push('Z')?;
        }
      }
    }
    Ok(string)
  }
}

#[cfg(test)]
mod tests {
  use crate::calendar::{DateTime, format::parse_bytes_into_tokens};

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
    let _0_tokens = parse_bytes_into_tokens(_0_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::parse(_0_DATA, _0_tokens.clone())
        .unwrap()
        .to_string::<32>(_0_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _0_DATA
    );
    let _1_tokens = parse_bytes_into_tokens(_1_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::parse(_1_DATA, _1_tokens.clone())
        .unwrap()
        .to_string::<32>(_1_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _1_DATA
    );
    let _2_tokens = parse_bytes_into_tokens(_2_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::parse(_2_DATA, _2_tokens.clone())
        .unwrap()
        .to_string::<32>(_2_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _2_DATA
    );
    let _3_tokens = parse_bytes_into_tokens(_3_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::parse(_3_DATA, _3_tokens.clone())
        .unwrap()
        .to_string::<32>(_3_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _3_DATA
    );
    let _4_tokens = parse_bytes_into_tokens(_4_FMT.iter().copied()).unwrap();
    assert_eq!(
      DateTime::parse(_4_DATA, _4_tokens.clone())
        .unwrap()
        .to_string::<32>(_4_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _4_DATA
    );
  }
}
