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

impl<NC, S, TP, WSB> RecievingTransport<TP> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
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

impl<NC, S, TP, WSB> SendingTransport<TP> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  TP: LeaseMut<WsParams>,
  WSB: LeaseMut<WebSocketBuffer>,
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
    send(pkg, pkgs_aux, self, cb).await?;
    Ok(())
  }
}

impl<NC, S, TP, WSB> Transport<TP> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
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
