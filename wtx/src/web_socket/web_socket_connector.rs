use crate::web_socket::WebSocketBuffer;
use core::array::IntoIter;
use httparse::Response;

/// WebSocket connector
#[derive(Debug)]
pub struct WebSocketConnector<C, H, R> {
  pub(crate) compression: C,
  pub(crate) headers: H,
  pub(crate) no_masking: bool,
  pub(crate) res_cb: R,
  pub(crate) wsb: WebSocketBuffer,
}

impl<C, H, R> WebSocketConnector<C, H, R> {
  /// Defaults to no compression.
  #[inline]
  pub fn set_compression<_C>(self, elem: _C) -> WebSocketConnector<_C, H, R> {
    WebSocketConnector {
      compression: elem,
      headers: self.headers,
      no_masking: self.no_masking,
      res_cb: self.res_cb,
      wsb: self.wsb,
    }
  }

  /// Additional header that must be sent in the request.
  #[inline]
  pub fn set_headers<_H>(self, elem: _H) -> WebSocketConnector<C, _H, R> {
    WebSocketConnector {
      compression: self.compression,
      headers: elem,
      no_masking: self.no_masking,
      res_cb: self.res_cb,
      wsb: self.wsb,
    }
  }

  /// If possible, stops the masking of frames.
  ///
  /// <https://datatracker.ietf.org/doc/draft-damjanovic-websockets-nomasking/>
  #[inline]
  #[must_use]
  pub const fn set_no_masking(mut self, elem: bool) -> WebSocketConnector<C, H, R> {
    self.no_masking = elem;
    self
  }

  /// Response callback.
  #[inline]
  pub fn set_res_cb<_R>(self, elem: _R) -> WebSocketConnector<C, H, _R> {
    WebSocketConnector {
      compression: self.compression,
      headers: self.headers,
      no_masking: self.no_masking,
      res_cb: elem,
      wsb: self.wsb,
    }
  }
}

impl Default
  for WebSocketConnector<
    (),
    IntoIter<(&'static str, &'static str), 0>,
    fn(&Response<'_, '_>) -> crate::Result<()>,
  >
{
  #[inline]
  fn default() -> Self {
    #[expect(clippy::unnecessary_wraps, reason = "false-positive")]
    #[inline]
    const fn res_cb(_: &Response<'_, '_>) -> crate::Result<()> {
      Ok(())
    }
    Self {
      compression: (),
      headers: [].into_iter(),
      no_masking: true,
      res_cb,
      wsb: WebSocketBuffer::new(),
    }
  }
}
