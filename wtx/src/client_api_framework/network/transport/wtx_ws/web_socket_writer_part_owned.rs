use crate::{
  client_api_framework::{
    network::{
      transport::{wtx_ws::send, SendingTransport, Transport},
      TransportGroup, WsParams,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{Lock, StreamWriter, Vector},
  web_socket::{
    compression::NegotiatedCompression, Frame, WebSocketCommonPartOwned, WebSocketWriterPartOwned,
  },
};

impl<C, DRSR, NC, SW> SendingTransport<DRSR> for WebSocketWriterPartOwned<C, NC, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
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

impl<C, DRSR, NC, SW> Transport<DRSR> for WebSocketWriterPartOwned<C, NC, SW, true>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, true>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = WsParams;
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
