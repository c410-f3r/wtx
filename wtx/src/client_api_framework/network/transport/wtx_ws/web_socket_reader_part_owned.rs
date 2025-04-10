use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{ReceivingTransport, Transport, wtx_ws::recv},
    },
    pkg::PkgsAux,
  },
  misc::{LeaseMut, Lock, Rng, StreamReader, StreamWriter},
  web_socket::{
    WebSocketCommonPartOwned, WebSocketReaderPartOwned, compression::NegotiatedCompression,
  },
};

impl<C, NC, R, SR, SW, TP> ReceivingTransport<TP> for WebSocketReaderPartOwned<C, NC, R, SR, true>
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
    _: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    recv(self.read_frame().await?, pkgs_aux).await?;
    Ok(())
  }
}

impl<C, NC, R, SR, SW, TP> Transport<TP> for WebSocketReaderPartOwned<C, NC, R, SR, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, R, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}
