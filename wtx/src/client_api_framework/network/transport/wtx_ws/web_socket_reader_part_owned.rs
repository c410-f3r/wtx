use crate::{
  client_api_framework::{
    network::{
      transport::{wtx_ws::recv, RecievingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::PkgsAux,
    Api,
  },
  misc::{Lock, StreamReader, StreamWriter},
  web_socket::{
    compression::NegotiatedCompression, WebSocketCommonPartOwned, WebSocketReaderPartOwned,
  },
};
use core::ops::Range;

impl<C, DRSR, NC, SR, SW> RecievingTransport<DRSR> for WebSocketReaderPartOwned<C, NC, SR, true>
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
    Ok(recv(self.read_frame().await?, pkgs_aux).await?)
  }
}

impl<C, DRSR, NC, SR, SW> Transport<DRSR> for WebSocketReaderPartOwned<C, NC, SR, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = WsParams;
}
