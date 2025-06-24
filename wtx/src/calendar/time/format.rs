use crate::{
  calendar::{CalendarError, CalendarToken, Time, Utc, format::parsed_data::ParsedData},
  collection::ArrayString,
  de::u32_string,
};

impl Time {
  /// Parses a sequence of bytes according to the specified tokens.
  ///
  /// See [`CalendarToken`] for more information.
  #[inline]
  pub fn parse(
    bytes: &[u8],
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<Self> {
    let ParsedData::<Utc>::Time(elem) = ParsedData::new(bytes, tokens)? else {
      return Err(CalendarError::InvalidParsingClockTime.into());
    };
    Ok(elem)
  }

  /// Creates a string represented based on the given `tokens`.
  ///
  /// A string of 18 bytes is usually more than enough for most representations.
  ///
  /// See [`CalendarToken`] for more information.
  #[inline]
  pub fn to_string<const N: usize>(
    &self,
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<ArrayString<N>> {
    let mut string = ArrayString::new();
    for token in tokens {
      match token {
        CalendarToken::Colon => {
          string.push(':')?;
        }
        CalendarToken::DotNano => {
          string.push('.')?;
          string.push_str(&u32_string(self.nanosecond().num()))?;
        }
        CalendarToken::TwoDigitHour => {
          string.push_str(self.hour().num_str())?;
        }
        CalendarToken::TwoDigitMinute => {
          string.push_str(self.minute().num_str())?;
        }
        CalendarToken::TwoDigitSecond => {
          string.push_str(self.second().num_str())?;
        }
        _ => return Err(CalendarError::InvalidParsingClockTime.into()),
      }
    }
    Ok(string)
  }
}

#[cfg(test)]
mod tests {
  use crate::calendar::{Time, format::parse_bytes_into_tokens};

  static _0_DATA: &[u8] = b"14:30:00";
  static _0_FMT: &[u8] = b"%H:%M:%S";

  static _1_DATA: &[u8] = b"23:40:20.123456789";
  static _1_FMT: &[u8] = b"%H:%M:%S%f?";

  #[test]
  fn parse_and_format() {
    let _0_tokens = parse_bytes_into_tokens(_0_FMT.iter().copied()).unwrap();
    assert_eq!(
      Time::parse(_0_DATA, _0_tokens.clone())
        .unwrap()
        .to_string::<18>(_0_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _0_DATA
    );
    let _1_tokens = parse_bytes_into_tokens(_1_FMT.iter().copied()).unwrap();
    assert_eq!(
      Time::parse(_1_DATA, _1_tokens.clone())
        .unwrap()
        .to_string::<18>(_1_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _1_DATA
    );
  }
}
