use crate::misc::{
  QueryWriter, _unlikely_dflt, bytes_pos1, bytes_rpos1, str_split_once1, ArrayString, FromRadix10,
  Lease,
};
use alloc::string::String;
use core::fmt::{Arguments, Debug, Display, Formatter, Write};

/// [Uri] with an owned array.
pub type UriArrayString<const N: usize> = Uri<ArrayString<N>>;
/// [Uri] with a string reference.
pub type UriRef<'uri> = Uri<&'uri str>;
/// [Uri] with an owned string.
pub type UriString = Uri<String>;

/// Elements that compose an URI.
///
/// ```txt
/// foo://user:password@hostname:80/path?query=value#hash
/// ```
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Uri<S>
where
  S: ?Sized,
{
  authority_start: u8,
  href_start: u8,
  initial_len: u16,
  port: Option<u16>,
  query_start: u16,
  uri: S,
}

impl<S> Uri<S>
where
  S: Lease<str>,
{
  #[inline]
  pub(crate) const fn _empty(uri: S) -> Self {
    Self { authority_start: 0, href_start: 0, initial_len: 0, port: None, query_start: 0, uri }
  }

  /// Analyzes the provided `uri` to create a new instance.
  #[inline]
  pub fn new(uri: S) -> Self {
    let (initial_len, authority_start, href_start, query_start) = Self::parts(uri.lease());
    let mut this = Self { authority_start, href_start, initial_len, port: None, query_start, uri };
    this.process_port();
    this
  }

  /// Full URI string
  #[inline]
  pub fn as_str(&self) -> &str {
    self.uri.lease()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.authority(), "user:password@hostname:80");
  /// ```
  #[inline]
  pub fn authority(&self) -> &str {
    self
      .uri
      .lease()
      .get(self.authority_start.into()..self.href_start.into())
      .unwrap_or_else(_unlikely_dflt)
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.host(), "hostname:80");
  /// ```
  #[inline]
  pub fn host(&self) -> &str {
    let authority = self.authority();
    if let Some(elem) = str_split_once1(authority, b'@') {
      elem.1
    } else {
      authority
    }
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.hostname(), "hostname");
  /// ```
  #[inline]
  pub fn hostname(&self) -> &str {
    let host = self.host();
    str_split_once1(host, b':').map_or(host, |el| el.0)
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2>
  ///
  /// Returns the hostname with a zeroed port if [`Self::port`] is [`Option::None`].
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.hostname_with_implied_port(), ("hostname", 80));
  /// ```
  #[inline]
  pub fn hostname_with_implied_port(&self) -> (&str, u16) {
    (self.hostname(), self.port().unwrap_or_default())
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.password(), "password");
  /// ```
  #[inline]
  pub fn password(&self) -> &str {
    if let Some(elem) = str_split_once1(self.userinfo(), b':') {
      elem.1
    } else {
      ""
    }
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.3>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.path(), "/path");
  /// ```
  #[inline]
  pub fn path(&self) -> &str {
    self.uri.lease().get(self.href_start.into()..self.query_start.into()).unwrap_or_default()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.3>
  ///
  /// Returns [`Option::None`] if the port couldn't be evaluated based on the schema or based on
  /// an explicit `... :SOME_NUMBER ...` declaration.
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.port(), Some(80));
  /// ```
  #[inline]
  pub fn port(&self) -> Option<u16> {
    self.port
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.4>
  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.5>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.query_and_fragment(), "?query=value#hash");
  /// ```
  #[inline]
  pub fn query_and_fragment(&self) -> &str {
    self.uri.lease().get(self.query_start.into()..).unwrap_or_default()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-4.2>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.relative_reference(), "/path?query=value#hash");
  /// ```
  #[inline]
  pub fn relative_reference(&self) -> &str {
    if let Some(elem) = self.uri.lease().get(self.href_start.into()..) {
      return elem;
    }
    ""
  }

  /// Like [`Self::relative_reference`] with the additional feature of returning `/` if empty.
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("");
  /// assert_eq!(uri.relative_reference_slash(), "/");
  /// ```
  #[inline]
  pub fn relative_reference_slash(&self) -> &str {
    let relative_reference = self.relative_reference();
    if relative_reference.is_empty() {
      "/"
    } else {
      relative_reference
    }
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.1>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.scheme(), "foo");
  /// ```
  #[inline]
  pub fn scheme(&self) -> &str {
    self
      .authority_start
      .checked_sub(3)
      .and_then(|index| self.uri.lease().get(..index.into()))
      .unwrap_or_default()
  }

  /// See [`UriRef`].
  #[inline]
  pub fn to_ref(&self) -> UriRef<'_> {
    UriRef {
      authority_start: self.authority_start,
      href_start: self.href_start,
      initial_len: self.initial_len,
      port: self.port,
      query_start: self.query_start,
      uri: self.uri.lease(),
    }
  }

  /// See [`UriString`].
  #[inline]
  pub fn to_string(&self) -> UriString {
    UriString {
      authority_start: self.authority_start,
      href_start: self.href_start,
      initial_len: self.initial_len,
      port: self.port,
      query_start: self.query_start,
      uri: self.uri.lease().into(),
    }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.user(), "user");
  /// ```
  #[inline]
  pub fn user(&self) -> &str {
    if let Some(elem) = str_split_once1(self.userinfo(), b':') {
      elem.0
    } else {
      ""
    }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.userinfo(), "user:password");
  /// ```
  #[inline]
  pub fn userinfo(&self) -> &str {
    if let Some(elem) = str_split_once1(self.authority(), b'@') {
      elem.0
    } else {
      ""
    }
  }

  fn parts(uri: &str) -> (u16, u8, u8, u16) {
    let initial_len = uri.len().try_into().unwrap_or(u16::MAX);
    let valid_uri = uri.get(..initial_len.into()).unwrap_or_default();
    let authority_start: u8 = valid_uri
      .match_indices("://")
      .next()
      .and_then(|(element, _)| element.wrapping_add(3).try_into().ok())
      .unwrap_or(0);
    let after_authority = valid_uri.as_bytes().get(authority_start.into()..).unwrap_or_default();
    let href_start = bytes_pos1(after_authority, b'/')
      .and_then(|idx| usize::from(authority_start).wrapping_add(idx).try_into().ok())
      .unwrap_or_else(|| initial_len.try_into().unwrap_or_default());
    let query_start = bytes_rpos1(valid_uri, b'?')
      .and_then(|element| element.try_into().ok())
      .unwrap_or(initial_len);
    (initial_len, authority_start, href_start, query_start)
  }

  #[inline]
  fn process_port(&mut self) {
    let uri = self.uri.lease().as_bytes();
    'explicit_port: {
      self.port = match uri.get(..self.href_start.into()) {
        Some([.., b':', a, b]) => u16::from_radix_10(&[*a, *b]).ok(),
        Some([.., b':', a, b, c]) => u16::from_radix_10(&[*a, *b, *c]).ok(),
        Some([.., b':', a, b, c, d]) => u16::from_radix_10(&[*a, *b, *c, *d]).ok(),
        Some([.., b':', a, b, c, d, e]) => u16::from_radix_10(&[*a, *b, *c, *d, *e]).ok(),
        _ => break 'explicit_port,
      };
      return;
    }
    self.port = match uri {
      [b'h', b't', b't', b'p', b's', ..] | [b'w', b's', b's', ..] => Some(443),
      [b'h', b't', b't', b'p', ..] | [b'w', b's', ..] => Some(80),
      [b'p', b'o', b's', b't', b'g', b'r', b'e', b's', b'q', b'l', ..]
      | [b'p', b'o', b's', b't', b'g', b'r', b'e', b's', ..] => Some(5432),
      _ => return,
    };
  }
}

impl UriString {
  /// Removes all content.
  #[inline]
  pub fn clear(&mut self) {
    let Self { authority_start, href_start, initial_len, port, query_start, uri } = self;
    *authority_start = 0;
    *href_start = 0;
    *initial_len = 0;
    *port = None;
    *query_start = 0;
    uri.clear();
  }

  /// Pushes an additional path only if there is no query.
  #[inline]
  pub fn push_path(&mut self, args: Arguments<'_>) -> crate::Result<()> {
    if !self.query_and_fragment().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    let prev = self.uri.len();
    self.uri.write_fmt(args)?;
    let diff = self.uri.len().wrapping_sub(prev);
    self.query_start = self.query_start.wrapping_add(diff.try_into().unwrap_or(u16::MAX));
    Ok(())
  }

  /// See [`QueryWriter<S>`].
  #[inline]
  pub fn query_writer(&mut self) -> crate::Result<QueryWriter<'_, String>> {
    if !self.query_and_fragment().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    Ok(QueryWriter::new(&mut self.uri))
  }

  /// Clears the internal storage and makes room for a new base URI.
  #[inline]
  pub fn reset(&mut self, uri: Arguments<'_>) -> crate::Result<()> {
    self.uri.clear();
    self.uri.write_fmt(uri)?;
    let (initial_len, authority_start, href_start, query_start) = Self::parts(&self.uri);
    self.authority_start = authority_start;
    self.href_start = href_start;
    self.initial_len = initial_len;
    self.query_start = query_start;
    Ok(())
  }

  /// Truncates the internal storage with the length of the base URI created in this instance.
  #[inline]
  pub fn truncate_with_initial_len(&mut self) {
    self.uri.truncate(self.initial_len.into());
    self.query_start = self.query_start.min(self.initial_len);
  }
}

impl<S> Debug for Uri<S>
where
  S: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    self.uri.fmt(f)
  }
}

impl<S> Display for Uri<S>
where
  S: Display,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    self.uri.fmt(f)
  }
}

impl<S> From<S> for Uri<S>
where
  S: Lease<str>,
{
  #[inline]
  fn from(value: S) -> Self {
    Self::new(value)
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::UriString;

  #[test]
  fn dynamic_methods_have_correct_behavior() {
    let mut uri = UriString::new("http://dasdas.com/rewqd".into());
    uri.push_path(format_args!("/tretre")).unwrap();
    assert_eq!(uri.path(), "/rewqd/tretre");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd/tretre");
    uri.truncate_with_initial_len();
    assert_eq!(uri.path(), "/rewqd");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd");
    uri.clear();
    assert_eq!(uri.path(), "");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "");
  }
}
