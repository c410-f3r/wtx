use crate::{
  client_api_framework::{
    Api,
    network::{
      TransportGroup, WsParams,
      transport::{
        SendingTransport, Transport,
        wtx_ws::{send_bytes, send_pkg},
      },
    },
    pkg::{Package, PkgsAux},
  },
  collections::Vector,
  misc::LeaseMut,
  net::StreamWriter,
  tls::TlsMode,
  web_socket::{Frame, WebSocketWriterOwned, web_socket_compression::NegotiatedWsCompression},
};

impl<NC, SW, TM, TP> SendingTransport<TP> for WebSocketWriterOwned<NC, SW, TM, true>
where
  NC: NegotiatedWsCompression,
  SW: StreamWriter,
  TM: TlsMode,
  TP: LeaseMut<WsParams>,
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

impl<NC, SW, TM, TP> Transport<TP> for WebSocketWriterOwned<NC, SW, TM, true>
where
  NC: NegotiatedWsCompression,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Inner = Self;
  type ReqId = ();
}

async fn cb<NC, SW, TM>(
  mut frame: Frame<&mut Vector<u8>>,
  trans: &mut WebSocketWriterOwned<NC, SW, TM, true>,
) -> crate::Result<()>
where
  NC: NegotiatedWsCompression,
  SW: StreamWriter,
  TM: TlsMode,
{
  trans.write_frame(&mut frame).await?;
  Ok(())
}
