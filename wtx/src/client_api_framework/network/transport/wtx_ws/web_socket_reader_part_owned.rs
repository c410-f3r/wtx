use crate::{
  client_api_framework::{
    network::{
      transport::{wtx_ws::recv, RecievingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::PkgsAux,
    Api,
  },
  misc::{LeaseMut, Lock, StreamReader, StreamWriter},
  web_socket::{
    compression::NegotiatedCompression, WebSocketCommonPartOwned, WebSocketReaderPartOwned,
  },
};
use core::ops::Range;

impl<C, NC, SR, SW, TP> RecievingTransport<TP> for WebSocketReaderPartOwned<C, NC, SR, true>
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
    Ok(recv(self.read_frame().await?, pkgs_aux).await?)
  }
}

impl<C, NC, SR, SW, TP> Transport<TP> for WebSocketReaderPartOwned<C, NC, SR, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
}
