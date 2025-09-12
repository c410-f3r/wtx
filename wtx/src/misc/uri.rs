use crate::{
  collection::ArrayStringU16,
  de::FromRadix10 as _,
  misc::{Lease, LeaseMut, bytes_pos1, bytes_rpos1, str_split_once1, str_split1},
};
use alloc::{boxed::Box, string::String};
use core::{
  fmt::{Arguments, Debug, Display, Formatter, Write},
  ops::{Deref, DerefMut},
};

/// [Uri] with an owned array.
pub type UriArrayString<const N: usize> = Uri<ArrayStringU16<N>>;
/// [Uri] with an owned string.
pub type UriBox = Uri<Box<str>>;
/// [Uri] with an owned string.
pub type UriCow<'uri> = Uri<alloc::borrow::Cow<'uri, str>>;
/// [Uri] with a string reference.
pub type UriRef<'uri> = Uri<&'uri str>;
/// [Uri] with a dynamic owned string.
pub type UriString = Uri<String>;

/// A Uniform Resource Identifier (URI) is a unique sequence of characters that identifies an
/// abstract or physical resource. This specific structure is used for identification purposes.
///
/// ```txt
/// foo://user:password@hostname:80/path?query=value#hash
/// ```
// \0\0\0 | foo:// | user:password@hostname:80 | /path | ?query=value | #hash |
//        |        |                           |       |              |       |
//        |        |                           |       |              |       |-> initial_len
//        |        |                           |       |              |
//        |        |                           |       |              |---------> fragment_start
//        |        |                           |       |
//        |        |                           |       |------------------------> query_start
//        |        |                           |
//        |        |                           |--------------------------------> href_start
//        |        |
//        |        |------------------------------------------------------------> authority_start
//        |
//        |---------------------------------------------------------------------> start
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Uri<S>
where
  S: ?Sized,
{
  authority_start: u8,
  fragment_start: u16,
  href_start: u8,
  initial_len: u16,
  port: Option<u16>,
  query_start: u16,
  start: u8,
  uri: S,
}

impl<S> Uri<S>
where
  S: Lease<str>,
{
  pub(crate) const fn empty(uri: S) -> Self {
    Self {
      authority_start: 0,
      fragment_start: 0,
      href_start: 0,
      initial_len: 0,
      port: None,
      query_start: 0,
      start: 0,
      uri,
    }
  }

  /// Analyzes the provided `uri` to create a new instance.
  #[inline]
  pub fn new(uri: S) -> Self {
    let mut this = Self::empty(uri);
    this.process();
    this
  }

  /// Full URI string
  #[inline]
  pub fn as_str(&self) -> &str {
    self.uri.lease().get(self.start.into()..).unwrap_or_default()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.authority(), "user:password@hostname:80");
  /// ```
  #[inline]
  pub fn authority(&self) -> &str {
    self.uri.lease().get(self.authority_start.into()..self.href_start.into()).unwrap_or_default()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.5>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.fragment(), "#hash");
  /// ```
  #[inline]
  pub fn fragment(&self) -> &str {
    self.uri.lease().get(self.fragment_start.into()..).unwrap_or_default()
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
    if let Some(elem) = str_split_once1(authority, b'@') { elem.1 } else { authority }
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

  /// Returns the number of characters.
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.len(), 53);
  /// ```
  #[inline]
  pub fn len(&self) -> usize {
    self.uri.lease().len()
  }

  /// <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1>
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.password(), "password");
  /// ```
  #[inline]
  pub fn password(&self) -> &str {
    if let Some(elem) = str_split_once1(self.userinfo(), b':') { elem.1 } else { "" }
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
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.query(), "?query=value");
  /// ```
  #[inline]
  pub fn query(&self) -> &str {
    self.uri.lease().get(self.query_start.into()..self.fragment_start.into()).unwrap_or_default()
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

  /// Iterator that returns the key/value pairs of a query, if any.
  ///
  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// let mut iter = uri.query_params();
  /// assert_eq!(iter.next(), Some(("query", "value")));
  /// assert_eq!(iter.next(), None);
  /// ```
  #[inline]
  pub fn query_params(&self) -> impl Iterator<Item = (&str, &str)> {
    let str = self.query().get(1..).unwrap_or_default();
    str_split1(str, b'&').filter_map(|el| str_split_once1(el, b'='))
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
    if relative_reference.is_empty() { "/" } else { relative_reference }
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
      .and_then(|idx| self.uri.lease().get(self.start.into()..idx.into()))
      .unwrap_or_default()
  }

  /// See [`UriRef`].
  #[inline]
  pub fn to_ref(&self) -> UriRef<'_> {
    UriRef {
      authority_start: self.authority_start,
      fragment_start: self.fragment_start,
      href_start: self.href_start,
      initial_len: self.initial_len,
      port: self.port,
      query_start: self.query_start,
      start: self.start,
      uri: self.uri.lease(),
    }
  }

  /// See [`UriString`].
  #[inline]
  pub fn to_string(&self) -> UriString {
    UriString {
      authority_start: self.authority_start,
      fragment_start: self.fragment_start,
      href_start: self.href_start,
      initial_len: self.initial_len,
      port: self.port,
      query_start: self.query_start,
      start: self.start,
      uri: self.uri.lease().into(),
    }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.user(), "user");
  /// ```
  #[inline]
  pub fn user(&self) -> &str {
    if let Some(elem) = str_split_once1(self.userinfo(), b':') { elem.0 } else { "" }
  }

  /// ```rust
  /// let uri = wtx::misc::Uri::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(uri.userinfo(), "user:password");
  /// ```
  #[inline]
  pub fn userinfo(&self) -> &str {
    if let Some(elem) = str_split_once1(self.authority(), b'@') { elem.0 } else { "" }
  }

  fn process(&mut self) {
    let Self {
      authority_start: this_authority_start,
      fragment_start: this_fragment_rstart,
      href_start: this_href_start,
      initial_len: this_initial_len,
      port: this_port,
      query_start: this_query_start,
      start: this_start,
      uri: this_uri,
    } = self;
    let indices = Self::process_indices(this_uri.lease());
    let (initial_len, start, authority_start, href_start, query_start, fragment_start) = indices;
    *this_authority_start = authority_start;
    *this_fragment_rstart = fragment_start;
    *this_href_start = href_start;
    *this_initial_len = initial_len;
    *this_port = Self::process_port(this_uri.lease().get(start.into()..self.href_start.into()));
    *this_query_start = query_start;
    *this_start = start;
  }

  fn process_indices(uri: &str) -> (u16, u8, u8, u8, u16, u16) {
    let initial_len = uri.len().try_into().unwrap_or(u16::MAX);
    let mut iter = uri.as_bytes().iter().copied().take(255);
    let init = if let Some(0) = iter.next() {
      let mut local_init: u8 = 1;
      for elem in iter {
        if elem != 0 {
          break;
        }
        local_init = local_init.wrapping_add(1);
      }
      local_init
    } else {
      0
    };
    let valid_uri = uri.get(..initial_len.into()).unwrap_or_default();
    let authority_start: u8 = valid_uri
      .match_indices("://")
      .next()
      .and_then(|(element, _)| element.wrapping_add(3).try_into().ok())
      .unwrap_or(0);
    let href_start = {
      let after_authority = valid_uri.get(authority_start.into()..).unwrap_or_default();
      bytes_pos1(after_authority, b'/')
        .and_then(|idx| usize::from(authority_start).wrapping_add(idx).try_into().ok())
        .unwrap_or_else(|| initial_len.try_into().unwrap_or_default())
    };
    let query_start = {
      let after_href = valid_uri.get(usize::from(href_start)..).unwrap_or_default();
      bytes_rpos1(after_href, b'?')
        .and_then(|idx| usize::from(href_start).wrapping_add(idx).try_into().ok())
        .unwrap_or(initial_len)
    };
    let fragment_start = {
      let after_path = uri.get(query_start.into()..).unwrap_or_default();
      bytes_rpos1(after_path, b'#')
        .and_then(|idx| usize::from(query_start).wrapping_add(idx).try_into().ok())
        .unwrap_or(initial_len)
    };
    (initial_len, init, authority_start, href_start, query_start, fragment_start)
  }

  fn process_port(str: Option<&str>) -> Option<u16> {
    match str.map(str::as_bytes) {
      Some([.., b':', a, b]) => u16::from_radix_10(&[*a, *b]).ok(),
      Some([.., b':', a, b, c]) => u16::from_radix_10(&[*a, *b, *c]).ok(),
      Some([.., b':', a, b, c, d]) => u16::from_radix_10(&[*a, *b, *c, *d]).ok(),
      Some([.., b':', a, b, c, d, e]) => u16::from_radix_10(&[*a, *b, *c, *d, *e]).ok(),
      Some([b'h', b't', b't', b'p', b's', ..] | [b'w', b's', b's', ..]) => Some(443),
      Some([b'h', b't', b't', b'p', ..] | [b'w', b's', ..]) => Some(80),
      Some([b'm', b'y', b's', b'q', b'l', ..]) => Some(3306),
      Some(
        [b'p', b'o', b's', b't', b'g', b'r', b'e', b's', b'q', b'l', ..]
        | [b'p', b'o', b's', b't', b'g', b'r', b'e', b's', ..],
      ) => Some(5432),
      _ => None,
    }
  }
}

impl UriString {
  /// Removes all content.
  #[inline]
  pub fn clear(&mut self) {
    let Self {
      authority_start,
      fragment_start,
      href_start,
      initial_len,
      port,
      query_start,
      start,
      uri,
    } = self;
    *authority_start = 0;
    *fragment_start = 0;
    *href_start = 0;
    *initial_len = 0;
    *port = None;
    *query_start = 0;
    *start = 0;
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
    let diff = self.uri.len().wrapping_sub(prev).try_into().unwrap_or(u16::MAX);
    self.query_start = self.query_start.wrapping_add(diff);
    self.fragment_start = self.fragment_start.wrapping_add(diff);
    Ok(())
  }

  /// Starts the query writer with an initial `?param=value`.
  #[inline]
  pub fn query_writer<ELEM>(
    &mut self,
    param: &str,
    value: ELEM,
  ) -> crate::Result<QueryWriter<'_, String>>
  where
    ELEM: Display,
  {
    if !self.query_and_fragment().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    QueryWriter { s: &mut self.uri }.do_write::<_, true>(param, value)
  }

  /// Starts the query writer with an initial `?param=value0(sep)value1(sep)value2...`.
  #[inline]
  pub fn query_writer_many<ELEM, SEP>(
    &mut self,
    param: &str,
    value: impl IntoIterator<Item = ELEM>,
    sep: SEP,
  ) -> crate::Result<QueryWriter<'_, String>>
  where
    ELEM: Display,
    SEP: Display,
  {
    if !self.query_and_fragment().is_empty() {
      return Err(crate::Error::UriCanNotBeOverwritten);
    }
    QueryWriter { s: &mut self.uri }.do_write_many::<_, _, true>(param, value, sep)
  }

  /// Clears the internal storage and makes room for a new base URI.
  #[inline]
  pub fn reset(&mut self) -> UriReset<'_, String> {
    self.uri.clear();
    UriReset(self)
  }

  /// Truncates the internal storage with the length of the base URI created in this instance.
  #[inline]
  pub fn truncate_with_initial_len(&mut self) {
    self.uri.truncate(self.initial_len.into());
    self.query_start = self.query_start.min(self.initial_len);
  }

  #[cfg(all(feature = "base64", feature = "http"))]
  pub(crate) fn buffer(
    &mut self,
    cb: impl FnOnce(&mut String) -> crate::Result<()>,
  ) -> crate::Result<()> {
    let idx = self.uri.len();
    let rslt = cb(&mut self.uri);
    self.uri.truncate(idx);
    rslt?;
    Ok(())
  }
}

impl<S> Lease<Uri<S>> for Uri<S> {
  #[inline]
  fn lease(&self) -> &Uri<S> {
    self
  }
}

impl<S> LeaseMut<Uri<S>> for Uri<S> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Uri<S> {
    self
  }
}

impl<S> Debug for Uri<S>
where
  S: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<S> Display for Uri<S>
where
  S: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
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

/// Returned by the [`Uri::reset`] method.
#[derive(Debug)]
pub struct UriReset<'uri, S>(&'uri mut Uri<S>)
where
  S: Lease<str>;

impl<S> Deref for UriReset<'_, S>
where
  S: Lease<str>,
{
  type Target = S;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0.uri
  }
}

impl<S> DerefMut for UriReset<'_, S>
where
  S: Lease<str>,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0.uri
  }
}

impl<S> Drop for UriReset<'_, S>
where
  S: Lease<str>,
{
  #[inline]
  fn drop(&mut self) {
    self.0.process();
  }
}

/// URL query writer.
#[derive(Debug)]
pub struct QueryWriter<'str, S> {
  s: &'str mut S,
}

impl<S> QueryWriter<'_, S>
where
  S: Lease<str> + Write,
{
  /// Writes `?param=value` or `&param=value`.
  #[inline]
  pub fn write<T>(self, param: &str, value: T) -> crate::Result<Self>
  where
    T: Display,
  {
    self.do_write::<_, false>(param, value)
  }

  /// Writes `?param=value0(sep)value1(sep)value2...` or `&param=value0(sep)value1(sep)value2...`.
  ///
  /// The separator (`sep`) will only be used if `value` is greater than one.
  #[inline]
  pub fn write_many<ELEM, SEP>(
    self,
    param: &str,
    value: impl IntoIterator<Item = ELEM>,
    sep: SEP,
  ) -> crate::Result<Self>
  where
    ELEM: Display,
    SEP: Display,
  {
    self.do_write_many::<_, _, false>(param, value, sep)
  }

  fn do_write<T, const IS_INITIAL: bool>(self, param: &str, value: T) -> crate::Result<Self>
  where
    T: Display,
  {
    if IS_INITIAL {
      self.s.write_fmt(format_args!("?{param}={value}"))?;
    } else {
      self.s.write_fmt(format_args!("&{param}={value}"))?;
    }
    Ok(self)
  }

  fn do_write_many<ELEM, SEP, const IS_INITIAL: bool>(
    self,
    param: &str,
    value: impl IntoIterator<Item = ELEM>,
    sep: SEP,
  ) -> crate::Result<Self>
  where
    ELEM: Display,
    SEP: Display,
  {
    let mut iter = value.into_iter();
    let Some(first) = iter.next() else {
      return Ok(self);
    };
    if IS_INITIAL {
      self.s.write_fmt(format_args!("?{param}={first}"))?;
    } else {
      self.s.write_fmt(format_args!("&{param}={first}"))?;
    }
    for elem in iter {
      self.s.write_fmt(format_args!("{sep}{elem}"))?;
    }
    Ok(self)
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::UriString;

  #[test]
  fn dynamic_methods_have_correct_behavior() {
    let mut uri = UriString::new("\0\0http://dasdas.com/rewqd".into());
    uri.push_path(format_args!("/tretre")).unwrap();
    assert_eq!(uri.scheme(), "http");
    assert_eq!(uri.path(), "/rewqd/tretre");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.fragment(), "");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd/tretre");
    uri.truncate_with_initial_len();
    assert_eq!(uri.scheme(), "http");
    assert_eq!(uri.path(), "/rewqd");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.fragment(), "");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "http://dasdas.com/rewqd");
    uri.clear();
    assert_eq!(uri.scheme(), "");
    assert_eq!(uri.path(), "");
    assert_eq!(uri.query(), "");
    assert_eq!(uri.fragment(), "");
    assert_eq!(uri.query_and_fragment(), "");
    assert_eq!(uri.as_str(), "");
  }
}
