use crate::web_socket::WebSocketBuffer;
use httparse::Request;

/// WebSocket acceptor
#[derive(Debug)]
pub struct WebSocketAcceptor<C, R> {
  pub(crate) compression: C,
  pub(crate) no_masking: bool,
  pub(crate) req: R,
  pub(crate) wsb: WebSocketBuffer,
}

impl<C, R> WebSocketAcceptor<C, R> {
  /// Defaults to no compression.
  #[inline]
  pub fn set_compression<_C>(self, elem: _C) -> WebSocketAcceptor<_C, R> {
    WebSocketAcceptor {
      compression: elem,
      no_masking: self.no_masking,
      req: self.req,
      wsb: self.wsb,
    }
  }

  /// If possible, stops the masking of frames.
  ///
  /// <https://datatracker.ietf.org/doc/draft-damjanovic-websockets-nomasking/>
  #[inline]
  #[must_use]
  pub const fn set_no_masking(mut self, elem: bool) -> WebSocketAcceptor<C, R> {
    self.no_masking = elem;
    self
  }

  /// Request callback.
  #[inline]
  pub fn set_req<_E, _R>(self, elem: _R) -> WebSocketAcceptor<C, _R>
  where
    _R: FnOnce(&Request<'_, '_>) -> Result<bool, _E>,
  {
    WebSocketAcceptor {
      compression: self.compression,
      no_masking: self.no_masking,
      req: elem,
      wsb: self.wsb,
    }
  }
}

impl Default for WebSocketAcceptor<(), fn(&Request<'_, '_>) -> crate::Result<bool>> {
  #[inline]
  fn default() -> Self {
    #[expect(clippy::unnecessary_wraps, reason = "false-positive")]
    #[inline]
    const fn req(_: &Request<'_, '_>) -> crate::Result<bool> {
      Ok(true)
    }
    Self { compression: (), no_masking: true, req, wsb: WebSocketBuffer::default() }
  }
}
