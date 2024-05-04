use crate::http::{Headers, ResponseData, StatusCode, Version};

/// Shortcut for mutable referenced data.
pub type ResponseMut<'data, D> = Response<&'data mut D>;
/// Shortcut for referenced data.
pub type ResponseRef<'data, D> = Response<&'data D>;

/// Represents the response from an HTTP request.
#[derive(Debug)]
pub struct Response<D> {
  /// See [ResponseData].
  pub data: D,
  /// See [StatusCode].
  pub status_code: StatusCode,
  /// See [Version].
  pub version: Version,
}

impl<D> Response<D>
where
  D: ResponseData,
{
  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub fn http2(data: D, status_code: StatusCode) -> Self {
    Self { data, status_code, version: Version::Http2 }
  }

  /// Shortcut to access the body of `data`.
  #[inline]
  pub fn body(&self) -> &D::Body {
    self.data.body()
  }

  /// Shortcut to access the headers of `data`.
  #[inline]
  pub fn headers(&self) -> &Headers {
    self.data.headers()
  }

  /// See [RequestRef].
  #[inline]
  pub fn to_ref(&self) -> ResponseRef<'_, D> {
    ResponseRef { data: &self.data, status_code: self.status_code, version: self.version }
  }
}
