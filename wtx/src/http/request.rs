use crate::http::{Method, Response, StatusCode, Version};

/// An HTTP request received by a server or to be sent by a client.
#[derive(Debug)]
pub struct Request<RRD> {
  /// See [`Method`].
  pub method: Method,
  /// See [`crate::http::ReqResData`].
  pub rrd: RRD,
  /// See [`Version`].
  pub version: Version,
}

impl<RRD> Request<RRD> {
  #[inline]
  pub(crate) fn _new(method: Method, rrd: RRD, version: Version) -> Self {
    Self { method, rrd, version }
  }

  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub fn http2(method: Method, rrd: RRD) -> Self {
    Self { method, rrd, version: Version::Http2 }
  }

  /// Creates a new [`Response`] using the inner buffer as well as the given `status_code`.
  #[inline]
  pub fn into_response(self, status_code: StatusCode) -> Response<RRD> {
    Response { rrd: self.rrd, status_code, version: self.version }
  }
}
