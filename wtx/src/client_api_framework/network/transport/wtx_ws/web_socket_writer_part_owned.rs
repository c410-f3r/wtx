use crate::{
  client_api_framework::{
    network::{
      transport::{wtx_ws::send, SendingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{LeaseMut, Lock, StreamWriter, Vector},
  web_socket::{
    compression::NegotiatedCompression, Frame, WebSocketCommonPartOwned, WebSocketWriterPartOwned,
  },
};

impl<C, NC, SW, TP> SendingTransport<TP> for WebSocketWriterPartOwned<C, NC, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
  TP: LeaseMut<WsParams>,
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

impl<C, NC, SW, TP> Transport<TP> for WebSocketWriterPartOwned<C, NC, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
}

async fn cb<C, NC, SW>(
  mut frame: Frame<&mut Vector<u8>, true>,
  trans: &mut WebSocketWriterPartOwned<C, NC, SW, true>,
) -> crate::Result<()>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
{
  trans.write_frame(&mut frame).await?;
  Ok(())
}
