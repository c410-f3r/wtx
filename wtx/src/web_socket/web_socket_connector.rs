use crate::web_socket::WebSocketBuffer;
use core::array::IntoIter;
use httparse::Response;

/// WebSocket connector
#[derive(Debug)]
pub struct WebSocketConnector<C, H, R, WB> {
  pub(crate) compression: C,
  pub(crate) headers: H,
  pub(crate) no_masking: bool,
  pub(crate) res_cb: R,
  pub(crate) wsb: WB,
}

impl<C, H, R, WB> WebSocketConnector<C, H, R, WB> {
  /// Defaults to no compression.
  #[inline]
  pub fn compression<NC>(self, elem: NC) -> WebSocketConnector<NC, H, R, WB> {
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
  pub fn headers<NH>(self, elem: NH) -> WebSocketConnector<C, NH, R, WB> {
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
  pub const fn no_masking(mut self, elem: bool) -> WebSocketConnector<C, H, R, WB> {
    self.no_masking = elem;
    self
  }

  /// Response callback.
  #[inline]
  pub fn res_cb<NR>(self, elem: NR) -> WebSocketConnector<C, H, NR, WB> {
    WebSocketConnector {
      compression: self.compression,
      headers: self.headers,
      no_masking: self.no_masking,
      res_cb: elem,
      wsb: self.wsb,
    }
  }

  /// WebSocket Buffer
  #[inline]
  pub fn wsb<NWSB>(self, elem: NWSB) -> WebSocketConnector<C, H, R, NWSB> {
    WebSocketConnector {
      compression: self.compression,
      headers: self.headers,
      no_masking: self.no_masking,
      res_cb: self.res_cb,
      wsb: elem,
    }
  }
}

impl Default
  for WebSocketConnector<
    (),
    IntoIter<(&'static str, &'static str), 0>,
    fn(&Response<'_, '_>) -> crate::Result<()>,
    WebSocketBuffer,
  >
{
  #[inline]
  fn default() -> Self {
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
