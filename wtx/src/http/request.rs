use crate::http::{Method, MsgDataMut, Response, StatusCode};

/// An HTTP request received by a server or to be sent by a client.
#[derive(Debug)]
pub struct Request<MD> {
  /// See [`Method`].
  pub method: Method,
  /// See [`crate::http::MsgData`].
  pub msg_data: MD,
}

impl<MD> Request<MD> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(method: Method, msg_data: MD) -> Self {
    Self { method, msg_data }
  }

  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub const fn http2(method: Method, msg_data: MD) -> Self {
    Self { method, msg_data }
  }

  /// Creates a new [`Response`] using the inner buffer as well as the given `status_code`.
  #[inline]
  pub fn into_response(self, status_code: StatusCode) -> Response<MD> {
    Response { msg_data: self.msg_data, status_code }
  }
}

impl<MD> Request<MD>
where
  MD: MsgDataMut,
{
  /// Clear body and header contents
  #[inline]
  pub fn clear(&mut self) {
    self.msg_data.clear();
  }
}
