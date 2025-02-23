mod web_socket;
mod web_socket_parts_owned;
mod web_socket_reader_part_owned;
mod web_socket_writer_part_owned;

use crate::{
  client_api_framework::{
    Api, ClientApiFrameworkError,
    misc::{
      manage_after_sending_bytes, manage_after_sending_pkg, manage_before_sending_bytes,
      manage_before_sending_pkg,
    },
    network::{
      WsParams, WsReqParamsTy,
      transport::{Transport, TransportParams},
    },
    pkg::{Package, PkgsAux},
  },
  misc::{LeaseMut, Vector},
  web_socket::{Frame, OpCode},
};
use core::ops::Range;

async fn recv<A, DRSR, TP>(
  frame: Frame<&mut [u8], true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> crate::Result<Range<usize>> {
  pkgs_aux.byte_buffer.clear();
  if let OpCode::Close = frame.op_code() {
    return Err(ClientApiFrameworkError::ClosedWsConnection.into());
  }
  pkgs_aux.byte_buffer.extend_from_copyable_slice(frame.payload())?;
  Ok(0..pkgs_aux.byte_buffer.len())
}

async fn send<A, AUX, DRSR, T, TP>(
  aux: &mut AUX,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
  before_sending: impl AsyncFnOnce(&mut AUX, &mut PkgsAux<A, DRSR, TP>, &mut T) -> Result<(), A::Error>,
  send: impl AsyncFnOnce(&mut AUX, &mut Vector<u8>, OpCode, &mut T) -> crate::Result<()>,
  after_sending: impl AsyncFnOnce(&mut AUX, &mut PkgsAux<A, DRSR, TP>, &mut T) -> Result<(), A::Error>,
) -> Result<(), A::Error>
where
  A: Api,
  T: Transport<TP>,
  TP: LeaseMut<WsParams>,
{
  before_sending(aux, pkgs_aux, &mut *trans).await?;
  let op_code = match pkgs_aux.tp.lease_mut().ext_req_params_mut().ty {
    WsReqParamsTy::Bytes => OpCode::Binary,
    WsReqParamsTy::String => OpCode::Text,
  };
  send(aux, &mut pkgs_aux.byte_buffer, op_code, trans).await?;
  pkgs_aux.byte_buffer.clear();
  after_sending(aux, pkgs_aux, &mut *trans).await?;
  pkgs_aux.tp.lease_mut().reset();
  Ok(())
}

async fn send_bytes<A, DRSR, T, TP>(
  mut bytes: &[u8],
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
  cb: impl AsyncFnOnce(Frame<&mut Vector<u8>, true>, &mut T) -> crate::Result<()>,
) -> Result<(), A::Error>
where
  A: Api,
  T: Transport<TP>,
  TP: LeaseMut<WsParams>,
{
  send(
    &mut bytes,
    pkgs_aux,
    trans,
    async move |aux, pa, tr| manage_before_sending_bytes(aux, pa, tr).await,
    async move |aux, buffer, op_code, trans| {
      buffer.extend_from_copyable_slice(aux)?;
      cb(Frame::new_fin(op_code, buffer), trans).await
    },
    async move |_, pa, _| manage_after_sending_bytes(pa).await,
  )
  .await
}

async fn send_pkg<A, DRSR, P, T, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
  cb: impl AsyncFnOnce(Frame<&mut Vector<u8>, true>, &mut T) -> crate::Result<()>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T, TP>,
  T: Transport<TP>,
  TP: LeaseMut<WsParams>,
{
  send(
    pkg,
    pkgs_aux,
    trans,
    async move |aux, pa, tr| manage_before_sending_pkg(aux, pa, tr).await,
    async move |_, buffer, op_code, trans| cb(Frame::new_fin(op_code, buffer), trans).await,
    async move |aux, pa, tr| manage_after_sending_pkg(aux, pa, tr).await,
  )
  .await
}
