use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{RecievingTransport, Transport, wtx_ws::recv},
    },
    pkg::PkgsAux,
  },
  misc::{LeaseMut, Lock, StreamReader, StreamWriter},
  web_socket::{
    WebSocketCommonPartOwned, WebSocketReaderPartOwned, compression::NegotiatedCompression,
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
