pub(crate) mod calendar_token;
pub(crate) mod parsed_data;
pub(crate) mod push;

use crate::{
  calendar::CalendarError,
  collection::{ArrayVector, ArrayVectorU8},
};

/// Parses a sequence of bytes into the corresponding tokens.
#[inline]
pub fn parse_bytes_into_tokens(
  bytes: impl IntoIterator<Item = u8>,
) -> crate::Result<ArrayVectorU8<calendar_token::CalendarToken, 16>> {
  let mut tokens = ArrayVector::new();
  let mut iter = bytes.into_iter().peekable();
  loop {
    let Some(first) = iter.next() else {
      break;
    };
    match first {
      b'%' => {
        let Some(second) = iter.next() else {
          return Err(CalendarError::InvalidParsingFormat.into());
        };
        match second {
          b'f' | b'z' => {
            let Some(third) = iter.next() else {
              return Err(CalendarError::InvalidParsingFormat.into());
            };
            tokens.push([second, third].try_into()?)?;
          }
          _ => {
            tokens.push([0, second].try_into()?)?;
          }
        }
      }
      b'G' => {
        let (Some(b'M'), Some(b'T')) = (iter.next(), iter.next()) else {
          return Err(CalendarError::InvalidParsingFormat.into());
        };
        tokens.push(calendar_token::CalendarToken::Gmt)?;
      }
      _ => {
        tokens.push([0, first].try_into()?)?;
      }
    }
  }
  Ok(tokens)
}
