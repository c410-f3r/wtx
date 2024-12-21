use crate::{
  client_api_framework::{
    network::{
      transport::{RecievingTransport, SendingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{Lock, StreamReader, StreamWriter},
  web_socket::{compression::NegotiatedCompression, WebSocketCommonPartOwned, WebSocketPartsOwned},
};
use core::ops::Range;

impl<C, DRSR, NC, SR, SW> RecievingTransport<DRSR> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  #[inline]
  async fn recv<A>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    self.reader.recv(pkgs_aux).await
  }
}

impl<C, DRSR, NC, SR, SW> SendingTransport<DRSR> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, WsParams>,
  {
    self.writer.send(pkg, pkgs_aux).await
  }
}

impl<C, DRSR, NC, SR, SW> Transport<DRSR> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = WsParams;
}
