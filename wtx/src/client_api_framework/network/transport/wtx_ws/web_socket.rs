use crate::{
  client_api_framework::{
    network::{
      transport::{
        wtx_ws::{recv, send},
        RecievingTransport, SendingTransport, Transport,
      },
      TransportGroup, WsParams,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{LeaseMut, Stream, Vector},
  web_socket::{compression::NegotiatedCompression, Frame, WebSocket, WebSocketBuffer},
};
use core::ops::Range;

impl<DRSR, NC, S, WSB> RecievingTransport<DRSR> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
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

impl<DRSR, NC, S, WSB> SendingTransport<DRSR> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
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
    send(pkg, pkgs_aux, self, cb).await?;
    Ok(())
  }
}

impl<DRSR, NC, S, WSB> Transport<DRSR> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = WsParams;
}

async fn cb<NC, S, WSB>(
  mut frame: Frame<&mut Vector<u8>, true>,
  trans: &mut WebSocket<NC, S, WSB, true>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  trans.write_frame(&mut frame).await?;
  Ok(())
}
