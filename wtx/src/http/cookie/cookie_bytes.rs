use crate::{
  http::{
    CookieError,
    cookie::{FMT1, FMT2, FMT3, FMT4, SameSite, cookie_generic::CookieGeneric, make_lowercase},
  },
  misc::{ArrayVector, FromRadix10, PercentDecode, Vector, bytes_split_once1, bytes_split1},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use core::{str, time::Duration};

/// A cookie is a small piece of data a server sends to a user's web browser.
#[derive(Debug)]
pub(crate) struct CookieBytes<'bytes> {
  pub(crate) generic: CookieGeneric<&'bytes [u8], &'bytes [u8]>,
}

impl<'bytes> CookieBytes<'bytes> {
  /// Creates a new instance based on a sequence of bytes received from a request.
  #[inline]
  pub(crate) fn parse<'local_bytes, 'vector>(
    bytes: &'local_bytes [u8],
    vector: &'vector mut Vector<u8>,
  ) -> crate::Result<Self>
  where
    'local_bytes: 'bytes,
    'vector: 'bytes,
  {
    let mut semicolons = bytes_split1(bytes, b';');

    let mut cookie: CookieGeneric<&'bytes [u8], &'bytes [u8]> = {
      let first_semicolon = semicolons.next().unwrap_or_default();
      let (name, value) = if let Some(elem) = bytes_split_once1(first_semicolon, b'=') {
        (elem.0.trim_ascii(), elem.1.trim_ascii())
      } else {
        return Err(crate::Error::from(CookieError::IrregularCookie));
      };
      if name.is_empty() {
        return Err(crate::Error::from(CookieError::MissingName));
      }
      let before_name_len = vector.len();
      let has_decoded_name = PercentDecode::new(name).decode(vector)?;
      let before_value_len = vector.len();
      let has_decoded_value = PercentDecode::new(value).decode(vector)?;
      CookieGeneric {
        domain: &[],
        expires: None,
        http_only: false,
        max_age: None,
        name: if has_decoded_name {
          vector.get(before_name_len..before_value_len).unwrap_or_default()
        } else {
          name
        },
        path: &[],
        same_site: None,
        secure: false,
        value: if has_decoded_value {
          vector.get(before_value_len..).unwrap_or_default()
        } else {
          value
        },
      }
    };

    let mut lower_case = ArrayVector::<u8, 12>::new();
    for semicolon in semicolons {
      let (name, value) = if let Some(elem) = bytes_split_once1(semicolon, b'=') {
        (elem.0.trim_ascii(), elem.1.trim_ascii())
      } else {
        return Err(crate::Error::from(CookieError::IrregularCookie));
      };
      make_lowercase::<12>(&mut lower_case, name);
      match (lower_case.as_ref(), value) {
        (b"domain", [_, ..]) => {
          cookie.domain = value;
        }
        (b"expires", [_, ..]) => {
          // SAFETY: `parse_from_str` will check the string content
          let str = unsafe { str::from_utf8_unchecked(value) };
          if let Ok(elem) = NaiveDateTime::parse_from_str(str, FMT1)
            .or_else(|_| NaiveDateTime::parse_from_str(str, FMT2))
            .or_else(|_| NaiveDateTime::parse_from_str(str, FMT3))
            .or_else(|_| NaiveDateTime::parse_from_str(str, FMT4))
            .map(|elem| DateTime::from_naive_utc_and_offset(elem, Utc))
          {
            cookie.expires = Some(elem)
          }
        }
        (b"httponly", _) => cookie.http_only = true,
        (b"max-age", [first, rest @ ..]) => {
          let is_negative = *first == b'-';
          let local_value = if is_negative { rest } else { value };
          if !local_value.iter().all(|el| el.is_ascii_digit()) {
            continue;
          }
          cookie.max_age = Some(if is_negative {
            Duration::ZERO
          } else {
            u64::from_radix_10(value)
              .map(Duration::from_secs)
              .unwrap_or_else(|_| Duration::from_secs(u64::MAX))
          })
        }
        (b"path", [_, ..]) => {
          cookie.path = value;
        }
        (b"samesite", [_, ..]) => {
          make_lowercase::<6>(&mut lower_case, value);
          match lower_case.as_ref() {
            b"lax" => cookie.same_site = Some(SameSite::Lax),
            b"none" => cookie.same_site = Some(SameSite::None),
            b"strict" => cookie.same_site = Some(SameSite::Strict),
            _ => {}
          }
        }
        (b"secure", _) => cookie.secure = true,
        _ => {}
      }
    }

    Ok(Self { generic: cookie })
  }
}
