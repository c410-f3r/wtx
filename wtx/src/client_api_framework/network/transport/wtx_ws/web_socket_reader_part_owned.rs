use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{ReceivingTransport, Transport, log_res},
    },
    pkg::PkgsAux,
  },
  collection::IndexedStorageMut,
  misc::LeaseMut,
  rng::Rng,
  stream::StreamReader,
  web_socket::{WebSocketReadMode, WebSocketReaderPartOwned, compression::NegotiatedCompression},
};

impl<NC, R, SR, TP> ReceivingTransport<TP> for WebSocketReaderPartOwned<NC, R, SR, true>
where
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
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
    pkgs_aux.byte_buffer.clear();
    let _frame = self.read_frame(&mut pkgs_aux.byte_buffer, WebSocketReadMode::Consistent).await?;
    log_res(pkgs_aux.log_body.1, &pkgs_aux.byte_buffer, TransportGroup::WebSocket);
    Ok(())
  }
}

impl<NC, R, SR, TP> Transport<TP> for WebSocketReaderPartOwned<NC, R, SR, true>
where
  NC: NegotiatedCompression,
  SR: StreamReader,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}
