use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{BiTransport, Transport, TransportParams},
      TransportGroup, WsParams, WsReqParamsTy,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{LeaseMut, Stream},
  rng::Rng,
  web_socket::{
    compression::NegotiatedCompression, FrameBufferVec, FrameBufferVecMut, FrameMutVec, OpCode,
    WebSocketBuffer, WebSocketClient,
  },
};
use core::ops::Range;

impl<DRSR, NC, RNG, S, WSB> Transport<DRSR> for (FrameBufferVec, WebSocketClient<NC, RNG, S, WSB>)
where
  NC: NegotiatedCompression,
  RNG: Rng,
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
    send(&mut self.0, pkg, pkgs_aux, &mut self.1).await
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
    send_recv(&mut self.0, pkg, pkgs_aux, &mut self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> BiTransport<DRSR> for (FrameBufferVec, WebSocketClient<NC, RNG, S, WSB>)
where
  NC: NegotiatedCompression,
  RNG: Rng,
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
    retrieve(pkgs_aux, &mut self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> Transport<DRSR>
  for (&mut FrameBufferVec, &mut WebSocketClient<NC, RNG, S, WSB>)
where
  NC: NegotiatedCompression,
  RNG: Rng,
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
    send(self.0, pkg, pkgs_aux, self.1).await
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
    send_recv(self.0, pkg, pkgs_aux, self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> BiTransport<DRSR>
  for (&mut FrameBufferVec, &mut WebSocketClient<NC, RNG, S, WSB>)
where
  NC: NegotiatedCompression,
  RNG: Rng,
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
    retrieve(pkgs_aux, self.1).await
  }
}

async fn retrieve<A, DRSR, NC, RNG, S, WSB>(
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> crate::Result<Range<usize>>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  pkgs_aux.byte_buffer.clear();
  let fb = &mut FrameBufferVecMut::from(&mut pkgs_aux.byte_buffer);
  let frame = ws.read_frame(fb).await?;
  if let OpCode::Close = frame.op_code() {
    return Err(crate::Error::CAF_ClosedWsConnection);
  }
  let indcs = frame.fb().indcs();
  Ok(indcs.1.into()..indcs.2)
}

async fn send<A, DRSR, NC, P, RNG, S, WSB>(
  fb: &mut FrameBufferVec,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> Result<(), A::Error>
where
  A: Api,
  NC: NegotiatedCompression,
  P: Package<A, DRSR, WsParams>,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  let mut trans = (fb, ws);
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut trans).await?;
  let op_code = match pkgs_aux.tp.ext_req_params_mut().ty {
    WsReqParamsTy::Bytes => OpCode::Binary,
    WsReqParamsTy::String => OpCode::Text,
  };
  let frame_rslt = FrameMutVec::new_fin(trans.0, op_code, &pkgs_aux.byte_buffer);
  trans.1.write_frame(&mut frame_rslt.map_err(Into::into)?).await.map_err(Into::into)?;
  pkgs_aux.byte_buffer.clear();
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}

async fn send_recv<A, DRSR, NC, P, RNG, S, WSB>(
  fb: &mut FrameBufferVec,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> Result<Range<usize>, A::Error>
where
  A: Api,
  NC: NegotiatedCompression,
  P: Package<A, DRSR, WsParams>,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  send(fb, pkg, pkgs_aux, ws).await?;
  let fb = &mut FrameBufferVecMut::from(&mut pkgs_aux.byte_buffer);
  let frame = ws.read_frame(fb).await.map_err(Into::into)?;
  if let OpCode::Close = frame.op_code() {
    return Err(crate::Error::CAF_ClosedWsConnection.into());
  }
  let indcs = frame.fb().indcs();
  Ok(indcs.1.into()..indcs.2)
}
