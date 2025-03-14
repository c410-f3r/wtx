use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    network::{
      TransportGroup, WsParams,
      transport::{
        ReceivingTransport, SendingTransport, Transport,
        wtx_ws::{recv, send_bytes, send_pkg},
      },
    },
    pkg::{Package, PkgsAux},
  },
  misc::{LeaseMut, Stream, Vector},
  web_socket::{Frame, WebSocket, WebSocketBuffer, compression::NegotiatedCompression},
};

impl<NC, S, TP, WSB> ReceivingTransport<TP> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
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

impl<NC, S, TP, WSB> SendingTransport<TP> for WebSocket<NC, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  TP: LeaseMut<WsParams>,
  WSB: LeaseMut<WebSocketBuffer>,
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
    send_bytes(bytes, pkgs_aux, self, cb).await
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
    send_pkg(pkg, pkgs_aux, self, cb).await
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
  type ReqId = ();
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
