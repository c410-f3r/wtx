use crate::{
  codec::PercentDecode,
  collections::{ArrayVectorU8, VectorUsize},
  http::cookie::CookieGeneric,
  misc::{AsciiGeneric, from_utf8_basic, str_split_once1, str_split1},
};
use core::str;

/// A version of [`CookieGeneric`] composed by string slices.
#[derive(Debug)]
pub struct CookieStr<'str>(
  /// See [`CookieGeneric`]
  pub CookieGeneric<&'str str>,
);

impl<'str> CookieStr<'str> {
  /// Creates a new instance based on a sequence of bytes received from a request.
  #[inline]
  pub fn parse<'local_str, 'vector>(
    buffer: &'vector mut VectorUsize<u8>,
    str: &'local_str str,
  ) -> crate::Result<Self>
  where
    'local_str: 'str,
    'vector: 'str,
  {
    let mut generic = CookieGeneric { values: ArrayVectorU8::new() };
    let mut parsed = ArrayVectorU8::<_, 8>::new();

    for semicolon in str_split1(str, AsciiGeneric::SEMICOLON) {
      let (name, value) = if let Some(elem) = str_split_once1(semicolon, AsciiGeneric::EQUAL) {
        (elem.0.trim_ascii(), elem.1.trim_ascii())
      } else {
        continue;
      };
      if name.is_empty() {
        continue;
      }

      let before_name_len = buffer.len().try_into()?;
      let has_decoded_name = PercentDecode::new(name.as_bytes()).decode(buffer)?;
      let name_src = if has_decoded_name {
        StrSrc::Buffer(before_name_len, buffer.len().try_into()?)
      } else {
        let (str_begin, str_end) = str_indcs(str, name)?;
        StrSrc::Str(str_begin, str_end)
      };

      let before_value_len = buffer.len().try_into()?;
      let has_decoded_value = PercentDecode::new(value.as_bytes()).decode(buffer)?;
      let value_src = if has_decoded_value {
        StrSrc::Buffer(before_value_len, buffer.len().try_into()?)
      } else {
        let (str_begin, str_end) = str_indcs(str, value)?;
        StrSrc::Str(str_begin, str_end)
      };

      parsed.push((name_src, value_src))?;
    }

    for (name_src, value_src) in parsed {
      let name = name_src.resolve(buffer, str)?;
      let value = value_src.resolve(buffer, str)?;
      generic.values.push((name.try_into()?, value))?;
    }

    Ok(CookieStr(generic))
  }
}

#[inline]
fn str_indcs(str: &str, sub_str: &str) -> crate::Result<(u16, u16)> {
  let str_begin: u16 = sub_str.as_ptr().addr().wrapping_sub(str.as_ptr().addr()).try_into()?;
  let str_end = str_begin.wrapping_add(sub_str.len().try_into()?);
  Ok((str_begin, str_end))
}

#[derive(Clone, Copy)]
enum StrSrc {
  Buffer(u16, u16),
  Str(u16, u16),
}

impl StrSrc {
  #[inline]
  fn resolve<'any>(self, buffer: &'any [u8], str: &'any str) -> crate::Result<&'any str> {
    match self {
      StrSrc::Buffer(begin, end) => {
        Ok(from_utf8_basic(buffer.get(usize::from(begin)..usize::from(end)).unwrap_or_default())?)
      }
      StrSrc::Str(begin, end) => {
        Ok(str.get(usize::from(begin)..usize::from(end)).unwrap_or_default())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{collections::VectorUsize, http::cookie::CookieStr};

  #[test]
  fn mixed() {
    let mut buffer = VectorUsize::new();
    let input = "plain1=regular_value; encoded=special%2Fvalue; plain2=foo";
    let cookie = CookieStr::parse(&mut buffer, input).unwrap();
    assert_eq!(cookie.0.values.len(), 3);
    assert_eq!(cookie.0.values[0], ("plain1".try_into().unwrap(), "regular_value"));
    assert_eq!(cookie.0.values[1], ("encoded".try_into().unwrap(), "special/value"));
    assert_eq!(cookie.0.values[2], ("plain2".try_into().unwrap(), "foo"));
  }
}
