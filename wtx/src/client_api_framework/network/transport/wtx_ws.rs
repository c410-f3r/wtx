mod web_socket;
mod web_socket_parts_owned;
mod web_socket_reader_part_owned;
mod web_socket_writer_part_owned;

use crate::{
  client_api_framework::{
    Api, ClientApiFrameworkError, SendBytesSource,
    misc::{
      _log_res, manage_after_sending_bytes, manage_after_sending_pkg, manage_before_sending_bytes,
      manage_before_sending_pkg,
    },
    network::{
      WsParams, WsReqParamsTy,
      transport::{Transport, TransportParams},
    },
    pkg::{Package, PkgsAux},
  },
  misc::{FnMutFut, LeaseMut, Vector},
  web_socket::{Frame, OpCode},
};

fn op_code<A, DRSR, TP>(pkgs_aux: &mut PkgsAux<A, DRSR, TP>) -> OpCode
where
  TP: LeaseMut<WsParams>,
{
  match pkgs_aux.tp.lease_mut().ext_req_params_mut().ty {
    WsReqParamsTy::Bytes => OpCode::Binary,
    WsReqParamsTy::String => OpCode::Text,
  }
}

async fn recv<A, DRSR, TP>(
  frame: Frame<&mut [u8], true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> crate::Result<()> {
  if let OpCode::Close = frame.op_code() {
    return Err(ClientApiFrameworkError::ClosedWsConnection.into());
  }
  pkgs_aux.byte_buffer.clear();
  pkgs_aux.byte_buffer.extend_from_copyable_slice(frame.payload())?;
  _log_res(&pkgs_aux.byte_buffer);
  Ok(())
}

async fn send_bytes<A, DRSR, T, TP>(
  bytes: SendBytesSource<'_>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
  mut cb: impl for<'any> FnMutFut<
    (Frame<&'any mut Vector<u8>, true>, &'any mut T),
    Result = crate::Result<()>,
  >,
) -> Result<(), A::Error>
where
  A: Api,
  T: Transport<TP>,
  TP: LeaseMut<WsParams>,
{
  manage_before_sending_bytes(bytes, pkgs_aux, trans).await?;
  if let SendBytesSource::Param(elem) = bytes {
    pkgs_aux.byte_buffer.extend_from_copyable_slice(elem)?;
  }
  cb.call((Frame::new_fin(op_code(pkgs_aux), &mut pkgs_aux.byte_buffer), trans)).await?;
  manage_after_sending_bytes(pkgs_aux).await?;
  Ok(())
}

async fn send_pkg<A, DRSR, P, T, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
  mut cb: impl for<'any> FnMutFut<
    (Frame<&'any mut Vector<u8>, true>, &'any mut T),
    Result = crate::Result<()>,
  >,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T, TP>,
  T: Transport<TP>,
  TP: LeaseMut<WsParams>,
{
  manage_before_sending_pkg(pkg, pkgs_aux, trans).await?;
  cb.call((Frame::new_fin(op_code(pkgs_aux), &mut pkgs_aux.byte_buffer), trans)).await?;
  manage_after_sending_pkg(pkg, pkgs_aux, trans).await
}
