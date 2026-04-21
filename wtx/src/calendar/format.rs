pub(crate) mod calendar_token;
pub(crate) mod parsed_data;
pub(crate) mod push;

use crate::{
  calendar::CalendarError,
  collection::{ArrayVector, ArrayVectorU8},
  misc::Lease,
};

/// Parses a sequence of bytes into the corresponding tokens.
#[inline]
pub fn parse_bytes_into_tokens<B>(
  bytes: impl IntoIterator<Item = B>,
) -> crate::Result<ArrayVectorU8<calendar_token::CalendarToken, 16>>
where
  B: Lease<u8>,
{
  let mut tokens = ArrayVector::new();
  let mut iter = bytes.into_iter();
  while let Some(first) = iter.next() {
    match first.lease() {
      b'%' => {
        let Some(second) = iter.next() else {
          return Err(CalendarError::InvalidParsingFormat.into());
        };
        match second.lease() {
          b'f' | b'z' => {
            let Some(third) = iter.next() else {
              return Err(CalendarError::InvalidParsingFormat.into());
            };
            tokens.push([*second.lease(), *third.lease()].try_into()?)?;
          }
          _ => {
            tokens.push([0, *second.lease()].try_into()?)?;
          }
        }
      }
      b'G' => {
        let (Some(b'M'), Some(b'T')) =
          (iter.next().map(|el| *el.lease()), iter.next().map(|el| *el.lease()))
        else {
          return Err(CalendarError::InvalidParsingFormat.into());
        };
        tokens.push(calendar_token::CalendarToken::Gmt)?;
      }
      _ => {
        tokens.push([0, *first.lease()].try_into()?)?;
      }
    }
  }
  Ok(tokens)
}
