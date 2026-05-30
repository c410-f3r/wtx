use crate::http::{Headers, MsgData, StatusCode};

/// Represents the response from an HTTP request.
#[derive(Debug)]
pub struct Response<MD> {
  /// See [`MsgData`].
  pub msg_data: MD,
  /// See [`StatusCode`].
  pub status_code: StatusCode,
}

impl<MD> Response<MD> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(msg_data: MD, status_code: StatusCode) -> Self {
    Self { msg_data, status_code }
  }

  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub const fn http2(data: MD, status_code: StatusCode) -> Self {
    Self { msg_data: data, status_code }
  }
}

impl<MD> Response<MD>
where
  MD: MsgData,
{
  /// Shortcut to access the body of `data`.
  #[inline]
  pub fn body(&self) -> &MD::Body {
    self.msg_data.body()
  }

  /// Shortcut to access the headers of `data`.
  #[inline]
  pub fn headers(&self) -> &Headers {
    self.msg_data.headers()
  }
}
