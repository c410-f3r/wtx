use crate::{
  misc::{simple_seed, Xorshift64},
  web_socket::WebSocketBuffer,
};
use httparse::Request;

/// WebSocket acceptor
#[derive(Debug)]
pub struct WebSocketAcceptor<C, R, WSB> {
  pub(crate) compression: C,
  pub(crate) no_masking: bool,
  pub(crate) req: R,
  pub(crate) rng: Xorshift64,
  pub(crate) wsb: WSB,
}

impl<C, R, WSB> WebSocketAcceptor<C, R, WSB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketAcceptor<NC, R, WSB> {
    WebSocketAcceptor {
      compression: elem,
      no_masking: self.no_masking,
      req: self.req,
      rng: self.rng,
      wsb: self.wsb,
    }
  }

  /// If possible, stops the masking of frames.
  ///
  /// <https://datatracker.ietf.org/doc/draft-damjanovic-websockets-nomasking/>
  #[inline]
  pub fn no_masking(mut self, elem: bool) -> WebSocketAcceptor<C, R, WSB> {
    self.no_masking = elem;
    self
  }

  /// Request callback.
  #[inline]
  pub fn req<NR>(self, elem: NR) -> WebSocketAcceptor<C, NR, WSB> {
    WebSocketAcceptor {
      compression: self.compression,
      no_masking: self.no_masking,
      req: elem,
      rng: self.rng,
      wsb: self.wsb,
    }
  }

  /// Random number generator
  #[inline]
  pub fn rng(mut self, elem: Xorshift64) -> WebSocketAcceptor<C, R, WSB> {
    self.rng = elem;
    self
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketAcceptor<C, R, NWSB> {
    WebSocketAcceptor {
      compression: self.compression,
      no_masking: self.no_masking,
      req: self.req,
      rng: self.rng,
      wsb: elem,
    }
  }
}

impl Default
  for WebSocketAcceptor<(), fn(&Request<'_, '_>) -> Result<(), crate::Error>, WebSocketBuffer>
{
  #[inline]
  fn default() -> Self {
    #[inline]
    fn req(_: &Request<'_, '_>) -> Result<(), crate::Error> {
      Ok(())
    }
    Self {
      compression: (),
      no_masking: true,
      req,
      rng: Xorshift64::from(simple_seed()),
      wsb: WebSocketBuffer::new(),
    }
  }
}
