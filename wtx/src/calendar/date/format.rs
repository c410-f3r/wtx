use crate::{
  calendar::{
    CalendarError, CalendarToken, Date, Utc,
    format::{
      parsed_data::ParsedData,
      push::{push_four_digit_year, push_two_space_day},
    },
  },
  collection::{ArrayString, ArrayStringU8},
  de::i16_string,
};

impl Date {
  /// Parses a sequence of bytes according to the specified tokens.
  ///
  /// See [`CalendarToken`] for more information.
  #[inline]
  pub fn parse(
    bytes: &[u8],
    tokens: impl IntoIterator<Item = CalendarToken>,
  ) -> crate::Result<Self> {
    let ParsedData::<Utc>::Date(elem) = ParsedData::new(bytes, tokens)? else {
      return Err(CalendarError::InvalidParsingDate.into());
    };
    Ok(elem)
  }

  /// Creates a string representation based on the given `tokens`.
  ///
  /// A string of 20 bytes is usually more than enough for most representations.
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
          string.push_str(self.month().short_name())?;
        }
        CalendarToken::AbbreviatedWeekdayName => {
          string.push_str(self.weekday().short_name())?;
        }
        CalendarToken::Comma => {
          string.push(',')?;
        }
        CalendarToken::Dash => {
          string.push('-')?;
        }
        CalendarToken::FourDigitYear => {
          push_four_digit_year(*self, &mut string)?;
        }
        CalendarToken::FullWeekdayName => {
          string.push_str(self.weekday().name())?;
        }
        CalendarToken::Slash => {
          string.push('/')?;
        }
        CalendarToken::Space => {
          string.push(' ')?;
        }
        CalendarToken::TwoDigitDay => {
          string.push_str(self.day().num_str())?;
        }
        CalendarToken::TwoDigitMonth => {
          string.push_str(self.month().num_str())?;
        }
        CalendarToken::TwoDigitYear => {
          string.push_str(&i16_string(self.year().num().rem_euclid(100)))?;
        }
        CalendarToken::TwoSpaceDay => {
          push_two_space_day(*self, &mut string)?;
        }
        _ => return Err(CalendarError::InvalidParsingDate.into()),
      }
    }
    Ok(string)
  }
}

#[cfg(test)]
mod tests {
  use crate::calendar::{Date, format::parse_bytes_into_tokens};

  static _0_DATA: &[u8] = b"Mon, 12 May 2025";
  static _0_FMT: &[u8] = b"%a, %d %b %Y";

  static _1_DATA: &[u8] = b"Monday, 12-May-25";
  static _1_FMT: &[u8] = b"%A, %d-%b-%y";

  static _2_DATA: &[u8] = b"Mon, 12-May-2025";
  static _2_FMT: &[u8] = b"%a, %d-%b-%Y";

  static _3_DATA: &[u8] = b"1999-02-03";
  static _3_FMT: &[u8] = b"%Y-%m-%d";

  #[test]
  fn parse_and_format() {
    let _0_tokens = parse_bytes_into_tokens(_0_FMT.iter().copied()).unwrap();
    assert_eq!(
      Date::parse(_0_DATA, _0_tokens.clone())
        .unwrap()
        .to_string::<32>(_0_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _0_DATA
    );
    let _1_tokens = parse_bytes_into_tokens(_1_FMT.iter().copied()).unwrap();
    assert_eq!(
      Date::parse(_1_DATA, _1_tokens.clone())
        .unwrap()
        .to_string::<32>(_1_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _1_DATA
    );
    let _2_tokens = parse_bytes_into_tokens(_2_FMT.iter().copied()).unwrap();
    assert_eq!(
      Date::parse(_2_DATA, _2_tokens.clone())
        .unwrap()
        .to_string::<32>(_2_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _2_DATA
    );
    let _3_tokens = parse_bytes_into_tokens(_3_FMT.iter().copied()).unwrap();
    assert_eq!(
      Date::parse(_3_DATA, _3_tokens.clone())
        .unwrap()
        .to_string::<32>(_3_tokens)
        .unwrap()
        .as_str()
        .as_bytes(),
      _3_DATA
    );
  }
}
