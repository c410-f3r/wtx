pub(crate) mod parsed_data;
pub(crate) mod push;
pub(crate) mod time_token;

use crate::{collection::ArrayVector, time::TimeError};

/// Parses a sequence of bytes into the corresponding tokens.
#[inline]
pub fn parse_bytes_into_tokens(
  bytes: impl IntoIterator<Item = u8>,
) -> crate::Result<ArrayVector<time_token::TimeToken, 16>> {
  let mut tokens = ArrayVector::new();
  let mut iter = bytes.into_iter().peekable();
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
        tokens.push(time_token::TimeToken::Gmt)?;
      }
      _ => {
        tokens.push([0, first].try_into()?)?;
      }
    }
  }
  Ok(tokens)
}
