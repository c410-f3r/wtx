use crate::http::{Headers, ReqResData, StatusCode, Version};

/// Represents the response from an HTTP request.
#[derive(Debug)]
pub struct Response<RRD> {
  /// See [`ReqResData`].
  pub rrd: RRD,
  /// See [`StatusCode`].
  pub status_code: StatusCode,
  /// See [`Version`].
  pub version: Version,
}

impl<RRD> Response<RRD> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(rrd: RRD, status_code: StatusCode, version: Version) -> Self {
    Self { rrd, status_code, version }
  }

  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub fn http2(data: RRD, status_code: StatusCode) -> Self {
    Self { rrd: data, status_code, version: Version::Http2 }
  }
}

impl<RRD> Response<RRD>
where
  RRD: ReqResData,
{
  /// Shortcut to access the body of `data`.
  #[inline]
  pub fn body(&self) -> &RRD::Body {
    self.rrd.body()
  }

  /// Shortcut to access the headers of `data`.
  #[inline]
  pub fn headers(&self) -> &Headers {
    self.rrd.headers()
  }
}
