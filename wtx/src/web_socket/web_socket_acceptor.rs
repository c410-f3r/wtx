use crate::{
  rng::{Xorshift64, simple_seed},
  web_socket::WebSocketBuffer,
};
use httparse::Request;

/// WebSocket acceptor
#[derive(Debug)]
pub struct WebSocketAcceptor<C, R, RNG, WSB> {
  pub(crate) compression: C,
  pub(crate) no_masking: bool,
  pub(crate) req: R,
  pub(crate) rng: RNG,
  pub(crate) wsb: WSB,
}

impl<C, R, RNG, WSB> WebSocketAcceptor<C, R, RNG, WSB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketAcceptor<NC, R, RNG, WSB> {
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
  pub const fn no_masking(mut self, elem: bool) -> WebSocketAcceptor<C, R, RNG, WSB> {
    self.no_masking = elem;
    self
  }

  /// Request callback.
  #[inline]
  pub fn req<NR>(self, elem: NR) -> WebSocketAcceptor<C, NR, RNG, WSB> {
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
  pub fn rng(mut self, elem: RNG) -> WebSocketAcceptor<C, R, RNG, WSB> {
    self.rng = elem;
    self
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketAcceptor<C, R, RNG, NWSB> {
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
  for WebSocketAcceptor<
    (),
    fn(&Request<'_, '_>) -> Result<(), crate::Error>,
    Xorshift64,
    WebSocketBuffer,
  >
{
  #[inline]
  fn default() -> Self {
    #[inline]
    const fn req(_: &Request<'_, '_>) -> Result<(), crate::Error> {
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
