use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    network::{
      TransportGroup, WsParams,
      transport::{
        SendingTransport, Transport,
        wtx_ws::{send_bytes, send_pkg},
      },
    },
    pkg::{Package, PkgsAux},
  },
  collection::Vector,
  misc::LeaseMut,
  rng::Rng,
  stream::StreamWriter,
  web_socket::{Frame, WebSocketWriterOwned, compression::NegotiatedCompression},
};

impl<NC, R, SW, TP> SendingTransport<TP> for WebSocketWriterOwned<NC, R, SW, true>
where
  NC: NegotiatedCompression,
  R: Rng,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
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

impl<NC, R, SW, TP> Transport<TP> for WebSocketWriterOwned<NC, R, SW, true>
where
  NC: NegotiatedCompression,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}

async fn cb<NC, R, SW>(
  mut frame: Frame<&mut Vector<u8>, true>,
  trans: &mut WebSocketWriterOwned<NC, R, SW, true>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  R: Rng,
  SW: StreamWriter,
{
  trans.write_frame(&mut frame).await?;
  Ok(())
}
