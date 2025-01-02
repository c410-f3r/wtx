use crate::{
  misc::{simple_seed, Xorshift64},
  web_socket::WebSocketBuffer,
};
use core::array::IntoIter;
use httparse::Response;

/// WebSocket connector
#[derive(Debug)]
pub struct WebSocketConnector<C, H, R, WSB> {
  pub(crate) compression: C,
  pub(crate) headers: H,
  pub(crate) no_masking: bool,
  pub(crate) res: R,
  pub(crate) rng: Xorshift64,
  pub(crate) wsb: WSB,
}

impl<C, H, R, WSB> WebSocketConnector<C, H, R, WSB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketConnector<NC, H, R, WSB> {
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
  pub fn headers<NH>(self, elem: NH) -> WebSocketConnector<C, NH, R, WSB> {
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
  pub fn no_masking(mut self, elem: bool) -> WebSocketConnector<C, H, R, WSB> {
    self.no_masking = elem;
    self
  }

  /// Response callback.
  #[inline]
  pub fn res<NR>(self, elem: NR) -> WebSocketConnector<C, H, NR, WSB> {
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
  pub fn rng(mut self, elem: Xorshift64) -> WebSocketConnector<C, H, R, WSB> {
    self.rng = elem;
    self
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketConnector<C, H, R, NWSB> {
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
