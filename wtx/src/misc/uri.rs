use crate::misc::{QueryWriter, _unlikely_dflt, str_rsplit_once1, str_split_once1};
use alloc::string::String;
use core::fmt::{Arguments, Debug, Formatter, Write};

/// [Uri] with a string reference.
pub type UriRef<'uri> = Uri<&'uri str>;
/// [Uri] with an owned string.
pub type UriString = Uri<String>;

/// Elements that compose an URI.
///
/// ```txt
/// foo://user:password@hostname:80/path?query=value#hash
/// ```
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Uri<S> {
  authority_start_idx: u16,
  href_start_idx: u16,
  initial_len: u16,
  uri: S,
}

impl<S> Uri<S>
where
  S: AsRef<str>,
{
  /// Analyzes the provided `uri` to create a new instance.
  #[inline]
  pub fn new(uri: S) -> Self {
    let initial_len = uri.as_ref().len().try_into().unwrap_or(u16::MAX);
    let valid_uri = uri.as_ref().get(..initial_len.into()).unwrap_or_else(_unlikely_dflt);
    let authority_start_idx: u16 = valid_uri
      .match_indices("://")
      .next()
      .and_then(|(element, _)| element.wrapping_add(3).try_into().ok())
      .unwrap_or_else(_unlikely_dflt);
    let href_start_idx = valid_uri
      .as_bytes()
      .iter()
      .copied()
      .enumerate()
      .skip(authority_start_idx.into())
      .find_map(|(idx, el)| (el == b'/').then_some(idx).and_then(|_usisze| _usisze.try_into().ok()))
      .unwrap_or(initial_len);
    Self { authority_start_idx, href_start_idx, initial_len, uri }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.authority(), "user:password@hostname:80");
  /// ```
  #[inline]
  pub fn authority(&self) -> &str {
    self
      .uri
      .as_ref()
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
    if let Some(elem) = self.uri.as_ref().get(self.href_start_idx.into()..) {
      if !elem.is_empty() {
        return elem;
      }
    }
    "/"
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
    if let Some((_, elem)) = str_rsplit_once1(before_hash, b'?') {
      elem
    } else {
      ""
    }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.schema(), "foo");
  /// ```
  #[inline]
  pub fn schema(&self) -> &str {
    let mut iter = self.uri.as_ref().split("://");
    let first_opt = iter.next();
    if iter.next().is_some() {
      first_opt.unwrap_or_else(_unlikely_dflt)
    } else {
      ""
    }
  }

  /// See [UriPartsRef].
  #[inline]
  pub fn to_ref(&self) -> UriRef<'_> {
    UriRef {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      initial_len: self.initial_len,
      uri: self.uri.as_ref(),
    }
  }

  /// See [UriPartsString].
  #[inline]
  pub fn to_string(&self) -> UriString {
    UriString {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      initial_len: self.initial_len,
      uri: self.uri.as_ref().into(),
    }
  }

  /// Full URI.
  #[inline]
  pub fn uri(&self) -> &str {
    self.uri.as_ref()
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
}

impl UriString {
  /// Clears the internal storage.
  #[inline]
  pub fn clear(&mut self) {
    self.authority_start_idx = 0;
    self.href_start_idx = 0;
    self.uri.clear();
  }

  /// Pushes an additional path erasing any subsequent content.
  #[inline]
  pub fn push_path(&mut self, args: Arguments<'_>) -> crate::Result<()> {
    if !self.query().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    self.uri.write_fmt(args)?;
    Ok(())
  }

  /// See [`QueryWriter<S>`].
  #[inline]
  pub fn query_writer(&mut self) -> crate::Result<QueryWriter<'_, String>> {
    if !self.query().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    Ok(QueryWriter::new(&mut self.uri))
  }

  /// Truncates the internal storage with the length of the URL initially created in this instance.
  ///
  /// If the current length is lesser than the original URL length, nothing will happen.
  #[inline]
  pub fn retain_with_initial_len(&mut self) {
    self.uri.truncate(self.initial_len.into());
  }
}

impl<S> Debug for Uri<S> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str("Uri")
  }
}

impl<S> From<S> for Uri<S>
where
  S: AsRef<str>,
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
    assert_eq!(uri.uri(), "http://dasdas.com/rewqd/tretre");
    uri.retain_with_initial_len();
    assert_eq!(uri.path(), "/rewqd");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.uri(), "http://dasdas.com/rewqd");
    uri.clear();
    assert_eq!(uri.path(), "/");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.uri(), "");
  }
}
