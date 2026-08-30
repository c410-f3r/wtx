// https://datatracker.ietf.org/doc/html/rfc7578

use crate::{
  collections::ArrayStringU8,
  futures::{FnFut, FnFutWrapper},
  http::{
    AutoStream, HttpError, Method, Mime, StatusCode,
    http2_server_framework::{
      Endpoint, Http2ServerFrameworkError, ResFinalizer, RouteMatch, State,
      misc::check_header_and_method,
    },
  },
  misc::{bytes_split_once_seq, from_utf8_basic},
};
use core::str;

/// A set of key/value pairs representing form fields and their values.
#[derive(Debug)]
pub struct FormData(
  /// Delimiter
  pub ArrayStringU8<48>,
);

impl<D, E, F, RES, S> Endpoint<D, E, S> for FnFutWrapper<(State<'_, D>, FormData), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(State<'any, D>, FormData), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let value = check_header_and_method(
      Mime::MultipartFormData,
      &auto_stream.req.msg_data.headers,
      auto_stream.req.method,
      Method::Post,
    )?;
    let (_, rhs) = value
      .split_once("boundary=")
      .ok_or_else(|| Http2ServerFrameworkError::FormDataHeaderIsMissingDelimiter.into())?;
    let form_data = FormData(rhs.trim().try_into()?);
    self
      .0
      .call((State::new(&mut auto_stream.data, &mut auto_stream.req), form_data))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

/// Parses the blocks of a `form/data` content
#[derive(Debug)]
pub struct FormDataIter<'any> {
  data: &'any [u8],
  delimiter: &'any str,
}

impl<'any> FormDataIter<'any> {
  /// New instance
  #[inline]
  pub fn new(data: &'any [u8], delimiter: &'any str) -> crate::Result<Self> {
    Ok(Self {
      data: bytes_split_once_seq(data, delimiter.as_bytes())
        .ok_or(HttpError::InvalidFormDataContent)?
        .1,
      delimiter,
    })
  }
}

impl<'any> Iterator for FormDataIter<'any> {
  type Item = crate::Result<FormDataBlock<'any>>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let Self { data, delimiter } = self;
    if let [b'-', b'-', ..] = data {
      return None;
    }
    let Some(block) = FormDataBlock::parse(data, delimiter) else {
      return Some(Err(HttpError::InvalidFormDataContent.into()));
    };
    Some(Ok(block))
  }
}

/// Delimited `form/data` data
#[derive(Debug, PartialEq)]
pub struct FormDataBlock<'any> {
  /// Content type
  pub content_type: Option<&'any str>,
  /// Filename
  pub filename: Option<&'any str>,
  /// Name
  pub name: &'any str,
  /// Value
  pub value: &'any [u8],
}

impl<'any> FormDataBlock<'any> {
  #[inline]
  fn parse(data: &mut &'any [u8], delimiter: &str) -> Option<Self> {
    let (name, filename) = consume_first_line(data)?;
    let mut content_type = None;
    consume_content_types(&mut content_type, data)?;
    let value = consume_value(data, delimiter)?;
    Some(Self {
      content_type: content_type.and_then(|el| from_utf8_basic(el).ok()),
      filename: filename.and_then(|el| from_utf8_basic(el).ok()),
      name: from_utf8_basic(name).ok()?,
      value,
    })
  }
}

#[inline]
fn consume_content(data: &[u8]) -> Option<&[u8]> {
  if let [b'c' | b'C', b'o', b'n', b't', b'e', b'n', b't', b'-', rest @ ..] = data {
    Some(rest)
  } else {
    None
  }
}

#[inline]
fn consume_content_types<'any>(
  content_type: &mut Option<&'any [u8]>,
  data: &mut &'any [u8],
) -> Option<()> {
  loop {
    if let [b'\r', b'\n', rest @ ..] = data {
      *data = rest;
      break;
    }
    *data = consume_content(data)?;
    let [b'T' | b't', b'y', b'p', b'e', b':', b' ', rest @ ..] = data else {
      let (_, rhs) = bytes_split_once_seq(data, b"\r\n")?;
      *data = rhs;
      continue;
    };
    let (lhs, rhs) = bytes_split_once_seq(rest, b"\r\n")?;
    *data = rhs;
    *content_type = Some(lhs);
  }
  Some(())
}

#[inline]
fn consume_first_line<'any>(data: &mut &'any [u8]) -> Option<(&'any [u8], Option<&'any [u8]>)> {
  *data = consume_until_alphanumeric(data)?;
  *data = consume_content(data)?;
  #[rustfmt::skip]
  let [
    b'D' | b'd', b'i', b's', b'p', b'o', b's', b'i', b't', b'i', b'o', b'n', b':',
    b' ', b'f', b'o', b'r', b'm', b'-', b'd', b'a', b't', b'a', b';',
    b' ', b'n', b'a', b'm', b'e', b'=', b'"', after_name @ ..
  ] = data else {
    return None;
  };
  let (name, after_name_value) = bytes_split_once_seq(after_name, b"\"")?;
  let filename = if let [b'\r', b'\n', bytes @ ..] = after_name_value {
    *data = bytes;
    None
  } else if let Some((_, after_fn)) = bytes_split_once_seq(after_name_value, b"; filename=\"") {
    let (filename, after_fn_value) = bytes_split_once_seq(after_fn, b"\"")?;
    *data = bytes_split_once_seq(after_fn_value, b"\r\n")?.1;
    Some(filename)
  } else {
    return None;
  };
  Some((name, filename))
}

#[inline]
fn consume_until_alphanumeric(data: &[u8]) -> Option<&[u8]> {
  let idx = data.iter().position(u8::is_ascii_alphanumeric)?;
  data.get(idx..)
}

#[inline]
fn consume_value<'any>(data: &mut &'any [u8], delimiter: &str) -> Option<&'any [u8]> {
  let (value, rest) = bytes_split_once_seq(data, delimiter.as_bytes())?;
  *data = rest;
  Some(if let [begin @ .., b'\r', b'\n', b'-', b'-'] = value { begin } else { value })
}

#[cfg(test)]
mod tests {
  use crate::http::http2_server_framework::{FormDataBlock, FormDataIter};

  #[test]
  fn case_insensitivity() {
    const PAYLOAD: &[u8] = b"\
      --foo\r\n\
      content-disposition: form-data; name=\"username\"\r\n\
      \r\n\
      john_doe\r\n\
      --foo--\
    ";

    let mut iter = FormDataIter::new(PAYLOAD, "foo").unwrap();
    assert_eq!(iter.next().unwrap().unwrap().name, "username");
  }

  #[test]
  fn example() {
    const PAYLOAD: &[u8] = b"
      ------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
      Content-Disposition: form-data; name=\"username\"\r\n\
      \r\n\
      john_doe\r\n\
      ------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
      Content-Disposition: form-data; name=\"profile_picture\"; filename=\"profile.jpg\"\r\n\
      Content-Type: image/jpeg\r\n\
      \r\n\
      [Binary data of the JPEG file]\r\n\
      ------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
      Content-Disposition: form-data; name=\"metadata\"\r\n\
      Content-Type: application/json\r\n\
      \r\n\
      {\"age\": 30, \"location\": \"New York\"}\r\n\
      ------WebKitFormBoundary7MA4YWxkTrZu0gW--
    ";

    let mut iter = FormDataIter::new(PAYLOAD, "----WebKitFormBoundary7MA4YWxkTrZu0gW").unwrap();
    assert_eq!(
      iter.next().unwrap().unwrap(),
      FormDataBlock { content_type: None, filename: None, name: "username", value: b"john_doe" }
    );
    assert_eq!(
      iter.next().unwrap().unwrap(),
      FormDataBlock {
        content_type: Some("image/jpeg"),
        filename: Some("profile.jpg"),
        name: "profile_picture",
        value: b"[Binary data of the JPEG file]"
      }
    );
    assert_eq!(
      iter.next().unwrap().unwrap(),
      FormDataBlock {
        content_type: Some("application/json"),
        filename: None,
        name: "metadata",
        value: br#"{"age": 30, "location": "New York"}"#
      }
    );
    assert!(iter.next().is_none());
  }

  #[test]
  fn ignored_headers() {
    const PAYLOAD: &[u8] = b"\
      --foo\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
      Content-Type: text/plain\r\n\
      Content-Transfer-Encoding: 8bit\r\n\
      \r\n\
      hello world\r\n\
      --foo--\
    ";

    let mut iter = FormDataIter::new(PAYLOAD, "foo").unwrap();
    assert_eq!(iter.next().unwrap().unwrap().content_type, Some("text/plain"));
  }

  #[test]
  fn multiline_value() {
    const PAYLOAD: &[u8] = b"\
      --foo\r\n\
      Content-Disposition: form-data; name=\"comment\"\r\n\
      \r\n\
      line1\r\n\
      line2\r\n\
      --foo--\
    ";

    let mut iter = FormDataIter::new(PAYLOAD, "foo").unwrap();
    let block = iter.next().unwrap().unwrap();
    assert_eq!(block.name, "comment");
    assert_eq!(block.value, b"line1\r\nline2");
  }

  #[test]
  fn trailing_parameters() {
    const PAYLOAD: &[u8] = b"\
      --foo\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"; size=11\r\n\
      \r\n\
      hello world\r\n\
      --foo--\
    ";

    let mut iter = FormDataIter::new(PAYLOAD, "foo").unwrap();
    assert_eq!(iter.next().unwrap().unwrap().filename, Some("test.txt"));
  }
}
