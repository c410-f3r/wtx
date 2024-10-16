use core::marker::PhantomData;

use crate::{
  http::{Headers, HttpError, KnownHeaderName},
  http2::{Http2Hook, Http2Params},
  misc::{LeaseMut, Rng, Stream},
  web_socket::{WebSocketBuffer, WebSocketServer},
};

#[derive(Debug)]
pub struct WebSocketHook<F, RNG, S, WSB> {
  phantom: PhantomData<(RNG, S, WSB)>,
  wsb: F,
}

impl<F, RNG, S, WSB> Http2Hook<false> for WebSocketHook<F, RNG, S, WSB>
where
  F: Fn() -> WSB,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  type Element = WebSocketServer<(), RNG, S, WSB>;

  #[inline]
  fn init(&mut self, headers: &Headers) -> crate::Result<Self::Element> {
    if !headers.get_by_name(b"sec-websocket-version").map_or(false, |el| el.value == b"13") {
      let expected = KnownHeaderName::SecWebsocketVersion;
      return Err(crate::Error::from(HttpError::MissingHeader(expected)).into());
    }
    //Ok(WebSocketServer::new((), rng, stream, wsb))
    todo!()
  }

  #[inline]
  fn http2_params(&mut self, hp: Http2Params) -> Http2Params {
    hp.set_enable_connect_protocol(true)
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
