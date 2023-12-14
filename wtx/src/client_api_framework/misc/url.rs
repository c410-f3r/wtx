use crate::client_api_framework::misc::QueryWriter;
use alloc::string::String;
use cl_aux::DynString;
use core::fmt::Arguments;

/// [Url] with a internal `String` storage.
pub type UrlString = Url<String>;

/// Some APIs must known certain parts of an URL in order to work.
///
/// Constructors do not verify if an URL has actual valid content.
#[derive(Debug)]
pub struct Url<S> {
  initial_len: usize,
  origin_end: usize,
  path_end: usize,
  url: S,
}

impl<S> Url<S>
where
  S: AsRef<str>,
{
  /// Creates all inner parts from an URL. For example, https://localhost/api_version/endpoint?foo=bar.
  #[inline]
  pub fn from_url(url: S) -> crate::Result<Self> {
    let [origin_end, path_end] = Self::instance_params_from_url(&url);
    Ok(Self { initial_len: url.as_ref().len(), origin_end, path_end, url })
  }

  /// For example, /api_version/endpoint?foo=bar
  #[inline]
  pub fn href(&self) -> &str {
    self.url.as_ref().get(self.origin_end..).unwrap_or_default()
  }

  /// For example, https://localhost
  #[inline]
  pub fn origin(&self) -> &str {
    self.url.as_ref().get(..self.origin_end).unwrap_or_default()
  }

  /// For example, /api_version/endpoint
  #[inline]
  pub fn path(&self) -> &str {
    self.url.as_ref().get(self.origin_end..self.path_end).unwrap_or_default()
  }

  /// For example, ?foo=bar
  #[inline]
  pub fn query(&self) -> &str {
    self.url.as_ref().get(self.path_end..).unwrap_or_default()
  }

  /// For example, https://localhost/api_version/endpoint
  #[inline]
  pub fn url(&self) -> &str {
    self.url.as_ref()
  }

  fn instance_params_from_url(url: &S) -> [usize; 2] {
    let mut slash_iter = url
      .as_ref()
      .as_bytes()
      .iter()
      .enumerate()
      .filter_map(|(idx, elem)| (*elem == b'/').then_some(idx));
    let _ = slash_iter.next();
    let _ = slash_iter.next();
    let Some(origin_end) = slash_iter.next() else {
      return [url.as_ref().len(); 2];
    };
    let after_origin = url.as_ref().get(origin_end..).unwrap_or_default();
    let path_end = if let Some(elem) = after_origin
      .as_bytes()
      .iter()
      .enumerate()
      .find_map(|(idx, elem)| (*elem == b'?').then_some(idx))
    {
      elem.wrapping_add(origin_end)
    } else {
      url.as_ref().len()
    };
    [origin_end, path_end]
  }
}

impl<S> Url<S>
where
  S: DynString,
{
  /// Creates all inner parts from an origin (https://localhost).
  #[inline]
  pub fn from_origin(origin_str: &str) -> crate::Result<Self> {
    Self::from_origin_path_and_query(origin_str, "", "")
  }

  /// Creates all inner parts from an origin (https://localhost) and a path (/api_version/endpoint).
  #[inline]
  pub fn from_origin_and_path(origin_str: &str, path_str: &str) -> crate::Result<Self> {
    Self::from_origin_path_and_query(origin_str, path_str, "")
  }

  /// Creates all inner parts from an origin (https://localhost), a path (/api_version/endpoint)
  /// and a query (?foo=bar)
  #[inline]
  pub fn from_origin_path_and_query(
    origin_str: &str,
    path_str: &str,
    query_str: &str,
  ) -> crate::Result<Self> {
    let origin_end = origin_str.len();
    let path_end = origin_str.len().wrapping_add(path_str.len());
    let url = {
      let mut s = S::default();
      s.push(origin_str)?;
      s.push(path_str)?;
      s.push(query_str)?;
      s
    };
    Ok(Self { initial_len: url.as_ref().len(), origin_end, path_end, url })
  }

  /// Clears the internal storage.
  #[inline]
  pub fn clear(&mut self) {
    self.url.clear();
    self.origin_end = 0;
    self.path_end = 0;
  }

  /// Pushes an additional path paths erasing any subsequent content.
  #[inline]
  pub fn push_path(&mut self, args: Arguments<'_>) -> crate::Result<()> {
    if self.url.as_ref().len() != self.path_end {
      return Err(crate::Error::UrlCanNotOverwriteInitiallySetUrl);
    }
    self.url.truncate(self.path_end);
    self.url.write_fmt(args)?;
    self.path_end = self.url.len();
    Ok(())
  }

  /// See [`QueryWriter<S>`].
  #[inline]
  pub fn query_writer(&mut self) -> crate::Result<QueryWriter<'_, S>> {
    if self.url.as_ref().len() != self.path_end {
      return Err(crate::Error::UrlCanNotOverwriteInitiallySetUrl);
    }
    Ok(QueryWriter::new(&mut self.url))
  }

  /// Erases any subsequent content after the origin.
  #[inline]
  pub fn retain_origin(&mut self) {
    self.url.truncate(self.origin_end);
    self.path_end = self.origin_end;
  }

  /// Truncates the internal storage with the length of the URL initially created in this instance.
  ///
  /// If the current length is lesser than the original URL length, nothing will happen.
  #[inline]
  pub fn retain_with_initial_len(&mut self) {
    if self.url.len() <= self.initial_len {
      return;
    }
    self.url.truncate(self.initial_len);
    let [origin_end, path_end] = Self::instance_params_from_url(&self.url);
    self.origin_end = origin_end;
    self.path_end = path_end;
  }

  /// Sets the origin if, and only if the inner length is equal to zero.
  #[inline]
  pub fn set_origin(&mut self, args: Arguments<'_>) -> crate::Result<()> {
    if !self.url.as_ref().is_empty() {
      return Err(crate::Error::UrlCanNotOverwriteInitiallySetUrl);
    }
    self.url.clear();
    self.url.write_fmt(args)?;
    self.origin_end = self.url.len();
    self.path_end = self.url.len();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::client_api_framework::misc::UrlString;

  #[test]
  fn from_origin_path_and_query_has_correct_behavior() {
    let url =
      UrlString::from_origin_path_and_query("http://ddas.com", "/wfdsq", "?fssqw=rq").unwrap();
    assert_eq!(url.origin(), "http://ddas.com");
    assert_eq!(url.path(), "/wfdsq");
    assert_eq!(url.query(), "?fssqw=rq");
    assert_eq!(url.url(), "http://ddas.com/wfdsq?fssqw=rq");
  }

  #[test]
  fn from_url_has_correct_behavior() {
    let url = UrlString::from_url("http://dasdas.com/rewqd?ter=w".into()).unwrap();
    assert_eq!(url.origin(), "http://dasdas.com");
    assert_eq!(url.path(), "/rewqd");
    assert_eq!(url.query(), "?ter=w");
    assert_eq!(url.url(), "http://dasdas.com/rewqd?ter=w");
  }

  #[test]
  fn mutable_methods_have_correct_behavior() {
    let mut url = UrlString::from_url("http://dasdas.com/rewqd".into()).unwrap();
    url.push_path(format_args!("/tretre")).unwrap();
    assert_eq!(url.origin(), "http://dasdas.com");
    assert_eq!(url.path(), "/rewqd/tretre");
    assert_eq!(url.query(), "");
    assert_eq!(url.url(), "http://dasdas.com/rewqd/tretre");

    url.retain_origin();
    assert_eq!(url.origin(), "http://dasdas.com");

    url.clear();
    url.set_origin(format_args!("http://tedfrwerew.com")).unwrap();
    assert_eq!(url.origin(), "http://tedfrwerew.com");
    assert_eq!(url.path(), "");
    assert_eq!(url.query(), "");
    assert_eq!(url.url(), "http://tedfrwerew.com");
  }
}
