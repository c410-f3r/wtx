use crate::{
  client_api_framework::{
    network::{
      transport::{RecievingTransport, SendingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{LeaseMut, Lock, StreamReader, StreamWriter},
  web_socket::{
    compression::NegotiatedCompression, WebSocketCommonPartOwned, WebSocketPartsOwned,
    WebSocketWriterPartOwned,
  },
};
use core::ops::Range;

impl<C, NC, SR, SW, TP> RecievingTransport<TP> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    self.reader.recv(pkgs_aux).await
  }
}

impl<C, NC, SR, SW, TP> SendingTransport<TP> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
{
  #[inline]
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    self.writer.send(pkg, pkgs_aux).await
  }
}

impl<C, NC, SR, SW, TP> Transport<TP> for WebSocketPartsOwned<C, NC, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = WebSocketWriterPartOwned<C, NC, SW, true>;
}
