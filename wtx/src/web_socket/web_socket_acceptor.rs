use crate::web_socket::WebSocketBuffer;
use httparse::Request;

/// WebSocket acceptor
#[derive(Debug)]
pub struct WebSocketAcceptor<C, R, WB> {
  pub(crate) compression: C,
  pub(crate) no_masking: bool,
  pub(crate) req: R,
  pub(crate) wsb: WB,
}

impl<C, R, WB> WebSocketAcceptor<C, R, WB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketAcceptor<NC, R, WB> {
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
  pub const fn no_masking(mut self, elem: bool) -> WebSocketAcceptor<C, R, WB> {
    self.no_masking = elem;
    self
  }

  /// Request callback.
  #[inline]
  pub fn req<NE, NR>(self, elem: NR) -> WebSocketAcceptor<C, NR, WB>
  where
    NR: FnOnce(&Request<'_, '_>) -> Result<bool, NE>,
  {
    WebSocketAcceptor {
      compression: self.compression,
      no_masking: self.no_masking,
      req: elem,
      wsb: self.wsb,
    }
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketAcceptor<C, R, NWSB> {
    WebSocketAcceptor {
      compression: self.compression,
      no_masking: self.no_masking,
      req: self.req,
      wsb: elem,
    }
  }
}

impl Default
  for WebSocketAcceptor<(), fn(&Request<'_, '_>) -> crate::Result<bool>, WebSocketBuffer>
{
  #[inline]
  fn default() -> Self {
    #[inline]
    const fn req(_: &Request<'_, '_>) -> crate::Result<bool> {
      Ok(true)
    }
    Self { compression: (), no_masking: true, req, wsb: WebSocketBuffer::default() }
  }
}
