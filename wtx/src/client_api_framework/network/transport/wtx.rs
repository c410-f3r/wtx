use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{BiTransport, Transport, TransportParams},
      TransportGroup, WsParams, WsReqParamsTy,
    },
    pkg::{Package, PkgsAux},
  },
  misc::{AsyncBounds, Stream},
  rng::Rng,
  web_socket::{
    compression::NegotiatedCompression, FrameBufferVec, FrameBufferVecMut, FrameMutVec, OpCode,
    WebSocketBuffer, WebSocketClient,
  },
};
use core::{borrow::BorrowMut, ops::Range};

impl<DRSR, NC, RNG, S, WSB> Transport<DRSR> for (FrameBufferVec, WebSocketClient<NC, RNG, S, WSB>)
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Params = WsParams;

  #[inline]
  async fn send<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<(), P::Error>
  where
    P: AsyncBounds + Package<DRSR, Self::Params>,
  {
    send(&mut self.0, pkg, pkgs_aux, &mut self.1).await
  }

  #[inline]
  async fn send_and_retrieve<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<Range<usize>, P::Error>
  where
    P: AsyncBounds + Package<DRSR, Self::Params>,
  {
    send_and_retrieve(&mut self.0, pkg, pkgs_aux, &mut self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> BiTransport<DRSR> for (FrameBufferVec, WebSocketClient<NC, RNG, S, WSB>)
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  #[inline]
  async fn retrieve<API>(
    &mut self,
    pkgs_aux: &mut PkgsAux<API, DRSR, Self::Params>,
  ) -> crate::Result<Range<usize>>
  where
    API: AsyncBounds,
  {
    retrieve(pkgs_aux, &mut self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> Transport<DRSR>
  for (&mut FrameBufferVec, &mut WebSocketClient<NC, RNG, S, WSB>)
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  const GROUP: TransportGroup = TransportGroup::WebSocket;
  type Params = WsParams;

  #[inline]
  async fn send<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<(), P::Error>
  where
    P: AsyncBounds + Package<DRSR, Self::Params>,
  {
    send(self.0, pkg, pkgs_aux, self.1).await
  }

  #[inline]
  async fn send_and_retrieve<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<Range<usize>, P::Error>
  where
    P: AsyncBounds + Package<DRSR, Self::Params>,
  {
    send_and_retrieve(self.0, pkg, pkgs_aux, self.1).await
  }
}

impl<DRSR, NC, RNG, S, WSB> BiTransport<DRSR>
  for (&mut FrameBufferVec, &mut WebSocketClient<NC, RNG, S, WSB>)
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  #[inline]
  async fn retrieve<API>(
    &mut self,
    pkgs_aux: &mut PkgsAux<API, DRSR, Self::Params>,
  ) -> crate::Result<Range<usize>>
  where
    API: AsyncBounds,
  {
    retrieve(pkgs_aux, self.1).await
  }
}

async fn retrieve<API, DRSR, NC, RNG, S, WSB>(
  pkgs_aux: &mut PkgsAux<API, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> crate::Result<Range<usize>>
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  pkgs_aux.byte_buffer.clear();
  let fb = &mut FrameBufferVecMut::from(&mut pkgs_aux.byte_buffer);
  let frame = ws.borrow_mut().read_frame(fb).await?;
  if let OpCode::Close = frame.op_code() {
    return Err(crate::Error::ClosedWsConnection.into());
  }
  let indcs = frame.fb().indcs();
  Ok(indcs.1.into()..indcs.2)
}

async fn send<DRSR, NC, P, RNG, S, WSB>(
  fb: &mut FrameBufferVec,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<P::Api, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> Result<(), P::Error>
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  P: AsyncBounds + Package<DRSR, WsParams>,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
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

async fn send_and_retrieve<DRSR, NC, P, RNG, S, WSB>(
  fb: &mut FrameBufferVec,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<P::Api, DRSR, WsParams>,
  ws: &mut WebSocketClient<NC, RNG, S, WSB>,
) -> Result<Range<usize>, P::Error>
where
  DRSR: AsyncBounds,
  NC: AsyncBounds + NegotiatedCompression,
  P: AsyncBounds + Package<DRSR, WsParams>,
  RNG: AsyncBounds + Rng,
  S: AsyncBounds + Stream,
  WSB: AsyncBounds + BorrowMut<WebSocketBuffer>,
  for<'ty> &'ty DRSR: AsyncBounds,
  for<'ty> &'ty NC: AsyncBounds,
  for<'ty> &'ty RNG: AsyncBounds,
  for<'ty> &'ty S: AsyncBounds,
  for<'ty> &'ty WSB: AsyncBounds,
{
  send(fb, pkg, pkgs_aux, ws).await?;
  let fb = &mut FrameBufferVecMut::from(&mut pkgs_aux.byte_buffer);
  let frame = ws.borrow_mut().read_frame(fb).await.map_err(Into::into)?;
  if let OpCode::Close = frame.op_code() {
    return Err(crate::Error::ClosedWsConnection.into());
  }
  let indcs = frame.fb().indcs();
  Ok(indcs.1.into()..indcs.2)
}
