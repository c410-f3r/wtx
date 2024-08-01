use crate::misc::{
  QueryWriter, _unlikely_dflt, str_rsplit_once1, str_split_once1, ArrayString, Lease,
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
  authority_start_idx: u8,
  href_start_idx: u16,
  initial_len: u16,
  uri: S,
}

impl<S> Uri<S>
where
  S: Lease<str>,
{
  #[inline]
  pub(crate) const fn _dummy(uri: S) -> Self {
    Self { authority_start_idx: 0, href_start_idx: 0, initial_len: 0, uri }
  }

  /// Creates a new instance based on the provided indexes.
  #[inline]
  pub fn from_parts(uri: S, authority_start_idx: u8, href_start_idx: u16) -> Self {
    let initial_len = uri.lease().len().try_into().unwrap_or(u16::MAX);
    Self { authority_start_idx, href_start_idx, initial_len, uri }
  }

  /// Analyzes the provided `uri` to create a new instance.
  #[inline]
  pub fn new(uri: S) -> Self {
    let (authority_start_idx, href_start_idx, initial_len) = Self::parts(uri.lease());
    Self { authority_start_idx, href_start_idx, initial_len, uri }
  }

  /// Full URI string
  #[inline]
  pub fn as_str(&self) -> &str {
    self.uri.lease()
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.authority_start_idx(), 6);
  /// ```
  #[inline]
  pub fn authority_start_idx(&self) -> u8 {
    self.authority_start_idx
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.authority(), "user:password@hostname:80");
  /// ```
  #[inline]
  pub fn authority(&self) -> &str {
    self
      .uri
      .lease()
      .get(self.authority_start_idx.into()..self.href_start_idx.into())
      .unwrap_or_else(_unlikely_dflt)
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.fragment(), "hash");
  /// ```
  #[inline]
  pub fn fragment(&self) -> &str {
    let href = self.href();
    let maybe_rslt = str_rsplit_once1(href, b'?').map_or(href, |el| el.1);
    if let Some((_, rslt)) = str_rsplit_once1(maybe_rslt, b'#') {
      rslt
    } else {
      maybe_rslt
    }
  }

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

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.hostname(), "hostname");
  /// ```
  #[inline]
  pub fn hostname(&self) -> &str {
    let host = self.host();
    str_split_once1(host, b':').map_or(host, |el| el.0)
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.href(), "/path?query=value#hash");
  /// ```
  #[inline]
  pub fn href(&self) -> &str {
    if let Some(elem) = self.uri.lease().get(self.href_start_idx.into()..) {
      if !elem.is_empty() {
        return elem;
      }
    }
    "/"
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.href_start_idx(), 31);
  /// ```
  #[inline]
  pub fn href_start_idx(&self) -> u16 {
    self.href_start_idx
  }

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

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.path(), "/path");
  /// ```
  #[inline]
  pub fn path(&self) -> &str {
    let href = self.href();
    str_rsplit_once1(href, b'?').map_or(href, |el| el.0)
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.port(), "80");
  /// ```
  #[inline]
  pub fn port(&self) -> &str {
    let host = self.host();
    str_split_once1(host, b':').map_or(host, |el| el.1)
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.query(), "query=value");
  /// ```
  #[inline]
  pub fn query(&self) -> &str {
    let href = self.href();
    let before_hash = if let Some((elem, _)) = str_rsplit_once1(href, b'#') { elem } else { href };
    str_rsplit_once1(before_hash, b'?').map(|el| el.1).unwrap_or_default()
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.schema(), "foo");
  /// ```
  #[inline]
  pub fn schema(&self) -> &str {
    self
      .authority_start_idx
      .checked_sub(3)
      .and_then(|index| self.uri.lease().get(..index.into()))
      .unwrap_or_default()
  }

  /// See [`UriRef`].
  #[inline]
  pub fn to_ref(&self) -> UriRef<'_> {
    UriRef {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      initial_len: self.initial_len,
      uri: self.uri.lease(),
    }
  }

  /// See [`UriString`].
  #[inline]
  pub fn to_string(&self) -> UriString {
    UriString {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      initial_len: self.initial_len,
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

  fn parts(uri: &str) -> (u8, u16, u16) {
    let initial_len = uri.len().try_into().unwrap_or(u16::MAX);
    let valid_uri = uri.get(..initial_len.into()).unwrap_or_else(_unlikely_dflt);
    let authority_start_idx: u8 = valid_uri
      .match_indices("://")
      .next()
      .and_then(|(element, _)| element.wrapping_add(3).try_into().ok())
      .unwrap_or(0);
    let href_start_idx = valid_uri
      .as_bytes()
      .iter()
      .copied()
      .enumerate()
      .skip(authority_start_idx.into())
      .find_map(|(idx, el)| (el == b'/').then_some(idx).and_then(|_usize| _usize.try_into().ok()))
      .unwrap_or(initial_len);
    (authority_start_idx, href_start_idx, initial_len)
  }
}

impl UriString {
  /// Pushes an additional path only if there is no query.
  #[inline]
  pub fn push_path(&mut self, args: Arguments<'_>) -> crate::Result<()> {
    if !self.query().is_empty() {
      return Err(crate::Error::MISC_UriCanNotBeOverwritten);
    }
    self.uri.write_fmt(args)?;
    Ok(())
  }

  /// See [`QueryWriter<S>`].
  #[inline]
  pub fn query_writer(&mut self) -> crate::Result<QueryWriter<'_, String>> {
    if !self.query().is_empty() {
      return Err(crate::Error::MISC_UriCanNotBeOverwritten);
    }
    Ok(QueryWriter::new(&mut self.uri))
  }

  /// Clears the internal storage and makes room for a new base URI.
  #[inline]
  pub fn reset(&mut self, uri: &str) {
    self.uri.clear();
    self.uri.push_str(uri);
    let (authority_start_idx, href_start_idx, initial_len) = Self::parts(uri);
    self.authority_start_idx = authority_start_idx;
    self.href_start_idx = href_start_idx;
    self.initial_len = initial_len;
  }

  /// Truncates the internal storage with the length of the base URI created in this instance.
  #[inline]
  pub fn truncate_with_initial_len(&mut self) {
    self.uri.truncate(self.initial_len.into());
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
  fn mutable_methods_have_correct_behavior() {
    let mut uri = UriString::new("http://dasdas.com/rewqd".into());
    uri.push_path(format_args!("/tretre")).unwrap();
    assert_eq!(uri.path(), "/rewqd/tretre");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd/tretre");
    uri.truncate_with_initial_len();
    assert_eq!(uri.path(), "/rewqd");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd");
    uri.reset("");
    assert_eq!(uri.path(), "/");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.as_str(), "");
  }
}
