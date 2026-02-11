use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{
        ReceivingTransport, SendingTransport, Transport, log_generic_res,
        wtx_ws::{send_bytes, send_pkg},
      },
    },
    pkg::{Package, PkgsAux},
  },
  collection::Vector,
  misc::LeaseMut,
  rng::Rng,
  stream::Stream,
  web_socket::{
    Frame, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin, compression::NegotiatedCompression,
  },
};

impl<NC, R, S, TP, WSB> ReceivingTransport<TP> for WebSocket<NC, R, S, WSB, true>
where
  NC: NegotiatedCompression,
  R: Rng,
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
    pkgs_aux.bytes_buffer.clear();
    let wpo = WebSocketPayloadOrigin::Consistent;
    let _frame = self.read_frame(&mut pkgs_aux.bytes_buffer, wpo).await?;
    log_generic_res(&pkgs_aux.bytes_buffer, pkgs_aux.log_data, TransportGroup::WebSocket);
    Ok(())
  }
}

impl<NC, R, S, TP, WSB> SendingTransport<TP> for WebSocket<NC, R, S, WSB, true>
where
  NC: NegotiatedCompression,
  R: Rng,
  S: Stream,
  TP: LeaseMut<WsParams>,
  WSB: LeaseMut<WebSocketBuffer>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: Option<&[u8]>,
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

impl<NC, R, S, TP, WSB> Transport<TP> for WebSocket<NC, R, S, WSB, true>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}

async fn cb<NC, R, S, WSB>(
  mut frame: Frame<&mut Vector<u8>, true>,
  trans: &mut WebSocket<NC, R, S, WSB, true>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  R: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  trans.write_frame(&mut frame).await?;
  Ok(())
}
