use alloc::string::String;

/// [UriParts] with an owned string.
pub type UriPartsString = UriParts<String>;
/// [UriParts] with a string reference.
pub type UriPartsRef<'uri> = UriParts<&'uri str>;

/// Elements that compose an URI.
///
/// ```txt
/// foo://user:password@hostname:80/path?query=value#hash
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UriParts<S> {
  authority_start_idx: usize,
  href_start_idx: usize,
  uri: S,
}

impl<S> UriParts<S>
where
  S: AsRef<str>,
{
  /// Analyzes the provided `uri` to create a new instance.
  #[inline]
  pub fn new(uri: S) -> Self {
    let authority_start_idx = uri
      .as_ref()
      .match_indices("://")
      .next()
      .map(|(element, _)| element.wrapping_add(3))
      .unwrap_or_default();
    let href_start_idx = uri
      .as_ref()
      .as_bytes()
      .iter()
      .copied()
      .enumerate()
      .skip(authority_start_idx)
      .find_map(|(idx, el)| (el == b'/').then_some(idx))
      .unwrap_or(uri.as_ref().len());
    Self { authority_start_idx, href_start_idx, uri }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.authority(), "user:password@hostname:80");
  /// ```
  #[inline]
  pub fn authority(&self) -> &str {
    self.uri.as_ref().get(self.authority_start_idx..self.href_start_idx).unwrap_or_default()
  }

  /// See [UriPartsRef].
  #[inline]
  pub fn as_ref(&self) -> UriPartsRef<'_> {
    UriPartsRef {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      uri: self.uri.as_ref(),
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.fragment(), "hash");
  /// ```
  #[inline]
  pub fn fragment(&self) -> &str {
    let href = self.href();
    let maybe_rslt = href.rsplit_once('?').map_or(href, |el| el.1);
    if let Some((_, rslt)) = maybe_rslt.rsplit_once('#') {
      rslt
    } else {
      maybe_rslt
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.host(), "hostname:80");
  /// ```
  #[inline]
  pub fn host(&self) -> &str {
    let authority = self.authority();
    if let Some(elem) = authority.split_once('@') {
      elem.1
    } else {
      authority
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.hostname(), "hostname");
  /// ```
  #[inline]
  pub fn hostname(&self) -> &str {
    let host = self.host();
    host.split_once(':').map_or(host, |el| el.0)
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.href(), "/path?query=value#hash");
  /// ```
  #[inline]
  pub fn href(&self) -> &str {
    if let Some(elem) = self.uri.as_ref().get(self.href_start_idx..) {
      if !elem.is_empty() {
        return elem;
      }
    }
    "/"
  }

  /// See [UriPartsString].
  #[inline]
  pub fn into_string(self) -> UriPartsString {
    UriPartsString {
      authority_start_idx: self.authority_start_idx,
      href_start_idx: self.href_start_idx,
      uri: self.uri.as_ref().into(),
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.password(), "password");
  /// ```
  #[inline]
  pub fn password(&self) -> &str {
    if let Some(elem) = self.userinfo().split_once(':') {
      elem.1
    } else {
      ""
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.path(), "/path");
  /// ```
  #[inline]
  pub fn path(&self) -> &str {
    let href = self.href();
    href.rsplit_once('?').map_or(href, |el| el.0)
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.port(), "80");
  /// ```
  #[inline]
  pub fn port(&self) -> &str {
    let host = self.host();
    host.split_once(':').map_or(host, |el| el.1)
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.query(), "query=value");
  /// ```
  #[inline]
  pub fn query(&self) -> &str {
    let href = self.href();
    let maybe_rslt = href.rsplit_once('?').map_or(href, |el| el.1);
    if let Some((rslt, _)) = maybe_rslt.rsplit_once('#') {
      rslt
    } else {
      maybe_rslt
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.schema(), "foo");
  /// ```
  #[inline]
  pub fn schema(&self) -> &str {
    let mut iter = self.uri.as_ref().split("://");
    let first_opt = iter.next();
    if iter.next().is_some() {
      first_opt.unwrap_or_default()
    } else {
      ""
    }
  }

  /// Full URI.
  #[inline]
  pub fn uri(&self) -> &str {
    self.uri.as_ref()
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.user(), "user");
  /// ```
  #[inline]
  pub fn user(&self) -> &str {
    if let Some(elem) = self.userinfo().split_once(':') {
      elem.0
    } else {
      ""
    }
  }

  /// ```rust
  /// let up = wtx::misc::UriParts::new("foo://user:password@hostname:80/path?query=value#hash");
  /// assert_eq!(up.userinfo(), "user:password");
  /// ```
  #[inline]
  pub fn userinfo(&self) -> &str {
    if let Some(elem) = self.authority().split_once('@') {
      elem.0
    } else {
      ""
    }
  }
}

impl<S> From<S> for UriParts<S>
where
  S: AsRef<str>,
{
  #[inline]
  fn from(value: S) -> Self {
    Self::new(value)
  }
}
