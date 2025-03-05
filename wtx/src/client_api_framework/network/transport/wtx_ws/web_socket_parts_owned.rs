use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{RecievingTransport, SendingTransport, Transport},
    },
    pkg::{Package, PkgsAux},
  },
  misc::{LeaseMut, Lock, StreamReader, StreamWriter},
  web_socket::{
    WebSocketCommonPartOwned, WebSocketPartsOwned, WebSocketWriterPartOwned,
    compression::NegotiatedCompression,
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
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: &[u8],
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    self.writer.send_bytes(bytes, pkgs_aux).await
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    self.writer.send_pkg(pkg, pkgs_aux).await
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
