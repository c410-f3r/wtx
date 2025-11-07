use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{ReceivingTransport, Transport, log_generic_res},
    },
    pkg::PkgsAux,
  },
  misc::LeaseMut,
  rng::Rng,
  stream::StreamReader,
  web_socket::{WebSocketPayloadOrigin, WebSocketReaderOwned, compression::NegotiatedCompression},
};

impl<NC, R, SR, TP> ReceivingTransport<TP> for WebSocketReaderOwned<NC, R, SR, true>
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
    pkgs_aux.bytes_buffer.clear();
    let _frame =
      self.read_frame(&mut pkgs_aux.bytes_buffer, WebSocketPayloadOrigin::Consistent).await?;
    log_generic_res(&pkgs_aux.bytes_buffer, pkgs_aux.log_body.1, TransportGroup::WebSocket);
    Ok(())
  }
}

impl<NC, R, SR, TP> Transport<TP> for WebSocketReaderOwned<NC, R, SR, true>
where
  NC: NegotiatedCompression,
  SR: StreamReader,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}
