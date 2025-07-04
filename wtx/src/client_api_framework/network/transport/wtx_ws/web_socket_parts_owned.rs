use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    network::{
      TransportGroup, WsParams,
      transport::{ReceivingTransport, SendingTransport, Transport},
    },
    pkg::{Package, PkgsAux},
  },
  misc::LeaseMut,
  rng::Rng,
  stream::{StreamReader, StreamWriter},
  sync::Lock,
  web_socket::{
    WebSocketCommonPartOwned, WebSocketPartsOwned, WebSocketWriterPartOwned,
    compression::NegotiatedCompression,
  },
};

impl<C, NC, R, SR, SW, TP> ReceivingTransport<TP> for WebSocketPartsOwned<C, NC, R, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, R, SW, true>>,
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    req_id: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    self.reader.recv(pkgs_aux, req_id).await
  }
}

impl<C, NC, R, SR, SW, TP> SendingTransport<TP> for WebSocketPartsOwned<C, NC, R, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, R, SW, true>>,
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: SendBytesSource<'_>,
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

impl<C, NC, R, SR, SW, TP> Transport<TP> for WebSocketPartsOwned<C, NC, R, SR, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, R, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = WebSocketWriterPartOwned<C, NC, R, SW, true>;
  type ReqId = ();
}
