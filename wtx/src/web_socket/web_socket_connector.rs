use crate::{
  misc::{Xorshift64, simple_seed},
  web_socket::WebSocketBuffer,
};
use core::array::IntoIter;
use httparse::Response;

/// WebSocket connector
#[derive(Debug)]
pub struct WebSocketConnector<C, H, R, RNG, WSB> {
  pub(crate) compression: C,
  pub(crate) headers: H,
  pub(crate) no_masking: bool,
  pub(crate) res: R,
  pub(crate) rng: RNG,
  pub(crate) wsb: WSB,
}

impl<C, H, R, RNG, WSB> WebSocketConnector<C, H, R, RNG, WSB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketConnector<NC, H, R, RNG, WSB> {
    WebSocketConnector {
      compression: elem,
      headers: self.headers,
      no_masking: self.no_masking,
      res: self.res,
      rng: self.rng,
      wsb: self.wsb,
    }
  }

  /// Additional header that must be sent in the request.
  #[inline]
  pub fn headers<NH>(self, elem: NH) -> WebSocketConnector<C, NH, R, RNG, WSB> {
    WebSocketConnector {
      compression: self.compression,
      headers: elem,
      no_masking: self.no_masking,
      res: self.res,
      rng: self.rng,
      wsb: self.wsb,
    }
  }

  /// If possible, stops the masking of frames.
  ///
  /// <https://datatracker.ietf.org/doc/draft-damjanovic-websockets-nomasking/>
  #[inline]
  pub fn no_masking(mut self, elem: bool) -> WebSocketConnector<C, H, R, RNG, WSB> {
    self.no_masking = elem;
    self
  }

  /// Response callback.
  #[inline]
  pub fn res<NR>(self, elem: NR) -> WebSocketConnector<C, H, NR, RNG, WSB> {
    WebSocketConnector {
      compression: self.compression,
      headers: self.headers,
      no_masking: self.no_masking,
      res: elem,
      rng: self.rng,
      wsb: self.wsb,
    }
  }

  /// Random number generator
  #[inline]
  pub fn rng(mut self, elem: RNG) -> WebSocketConnector<C, H, R, RNG, WSB> {
    self.rng = elem;
    self
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketConnector<C, H, R, RNG, NWSB> {
    WebSocketConnector {
      compression: self.compression,
      headers: self.headers,
      no_masking: self.no_masking,
      res: self.res,
      rng: self.rng,
      wsb: elem,
    }
  }
}

impl Default
  for WebSocketConnector<
    (),
    IntoIter<(&'static [u8], &'static [u8]), 0>,
    fn(&Response<'_, '_>) -> Result<(), crate::Error>,
    Xorshift64,
    WebSocketBuffer,
  >
{
  #[inline]
  fn default() -> Self {
    #[inline]
    fn res(_: &Response<'_, '_>) -> Result<(), crate::Error> {
      Ok(())
    }
    Self {
      compression: (),
      headers: [].into_iter(),
      no_masking: true,
      res,
      rng: Xorshift64::from(simple_seed()),
      wsb: WebSocketBuffer::new(),
    }
  }
}
