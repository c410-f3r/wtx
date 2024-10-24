use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{BiTransport, Transport, TransportParams},
      TransportGroup, WsParams, WsReqParamsTy,
    },
    pkg::{Package, PkgsAux},
    Api, ClientApiFrameworkError,
  },
  misc::{LeaseMut, Stream},
  web_socket::{
    compression::NegotiatedCompression, Frame, OpCode, WebSocketBuffer, WebSocketClient,
  },
};
use core::ops::Range;

impl<DRSR, NC, S, WSB> Transport<DRSR> for WebSocketClient<NC, S, WSB>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Params = WsParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    send(pkg, pkgs_aux, self).await
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    send_recv(pkg, pkgs_aux, self).await
  }
}

impl<DRSR, NC, S, WSB> BiTransport<DRSR> for WebSocketClient<NC, S, WSB>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  #[inline]
  async fn retrieve<A>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> crate::Result<Range<usize>>
  where
    A: Api,
  {
    retrieve(pkgs_aux, self).await
  }
}

async fn retrieve<A, DRSR, NC, S, WSB>(
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, S, WSB>,
) -> crate::Result<Range<usize>>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  pkgs_aux.byte_buffer.clear();
  let frame = ws.read_frame().await?;
  if let OpCode::Close = frame.op_code() {
    return Err(ClientApiFrameworkError::ClosedWsConnection.into());
  }
  pkgs_aux.byte_buffer.extend_from_copyable_slice(frame.payload())?;
  Ok(0..pkgs_aux.byte_buffer.len())
}

async fn send<A, DRSR, NC, P, S, WSB>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, S, WSB>,
) -> Result<(), A::Error>
where
  A: Api,
  NC: NegotiatedCompression,
  P: Package<A, DRSR, WsParams>,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *ws).await?;
  let op_code = match pkgs_aux.tp.ext_req_params_mut().ty {
    WsReqParamsTy::Bytes => OpCode::Binary,
    WsReqParamsTy::String => OpCode::Text,
  };
  let mut frame = Frame::new_fin(op_code, &mut pkgs_aux.byte_buffer);
  ws.write_frame(&mut frame).await.map_err(Into::into)?;
  pkgs_aux.byte_buffer.clear();
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}

async fn send_recv<A, DRSR, NC, P, S, WSB>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, S, WSB>,
) -> Result<Range<usize>, A::Error>
where
  A: Api,
  NC: NegotiatedCompression,
  P: Package<A, DRSR, WsParams>,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  send(pkg, pkgs_aux, ws).await?;
  let frame = ws.read_frame().await.map_err(Into::into)?;
  if let OpCode::Close = frame.op_code() {
    return Err(A::Error::from(ClientApiFrameworkError::ClosedWsConnection.into()));
  }
  pkgs_aux.byte_buffer.extend_from_copyable_slice(frame.payload()).map_err(Into::into)?;
  Ok(0..pkgs_aux.byte_buffer.len())
}
