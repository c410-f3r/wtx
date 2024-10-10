use crate::{http::Headers, http2::Http2Params};

/// Introspection points that generate elements.
///
/// The current interface is heavily influenced by RFC-8441.
pub trait Http2Hook<const IS_CLIENT: bool> {
  type Element;

  /// Initializes the hook element when a new stream is created.
  fn init(&mut self, headers: &Headers) -> crate::Result<Self::Element>;

  /// Modifies HTTP/2 parameters. Happens before [`Self::init`].
  fn http2_params(&mut self, hp: Http2Params) -> Http2Params;

  fn read_data(&mut self, data: &[u8], element: &mut Self::Element) -> crate::Result<()>;

  fn write_data(&mut self, data: &[u8], element: &mut Self::Element) -> crate::Result<()>;
}

impl<const IS_CLIENT: bool> Http2Hook<IS_CLIENT> for () {
  type Element = ();

  #[inline]
  fn init(&mut self, _: &Headers) -> crate::Result<Self::Element> {
    Ok(())
  }

  #[inline]
  fn http2_params(&mut self, hp: Http2Params) -> Http2Params {
    hp
  }

  #[inline]
  fn read_data(&mut self, _: &[u8], _: &mut Self::Element) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  fn write_data(&mut self, _: &[u8], _: &mut Self::Element) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "web-socket")]
mod websocket {
  use crate::{
    http::{Headers, HttpError, KnownHeaderName},
    http2::{Http2Hook, Http2Params},
    misc::{LeaseMut, Rng, Stream},
    web_socket::{WebSocketBuffer, WebSocketServer},
  };

  impl<RNG, S, WSB> Http2Hook<false> for WebSocketServer<(), RNG, S, WSB>
  where
    RNG: Rng,
    S: Stream,
    WSB: LeaseMut<WebSocketBuffer>,
  {
    type Element = ();

    #[inline]
    fn init(&mut self, headers: &Headers) -> crate::Result<Self::Element> {
      if !headers.get_by_name(b"sec-websocket-version").map_or(false, |el| el.value == b"13") {
        let expected = KnownHeaderName::SecWebsocketVersion;
        return Err(crate::Error::from(HttpError::MissingHeader(expected)).into());
      }
      Ok(())
    }

    #[inline]
    fn http2_params(&mut self, hp: Http2Params) -> Http2Params {
      hp.set_enable_connect_protocol(true)
    }

    #[inline]
    fn read_data(&mut self, a: &[u8], _: &mut Self::Element) -> crate::Result<()> {
      Ok(())
    }

    #[inline]
    fn write_data(&mut self, _: &[u8], _: &mut Self::Element) -> crate::Result<()> {
      Ok(())
    }
  }
}
