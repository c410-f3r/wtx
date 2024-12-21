mod web_socket;
mod web_socket_parts_owned;
mod web_socket_reader_part_owned;
mod web_socket_writer_part_owned;

use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{Transport, TransportParams},
      WsParams, WsReqParamsTy,
    },
    pkg::{Package, PkgsAux},
    Api, ClientApiFrameworkError,
  },
  misc::{FnMutFut, Vector},
  web_socket::{Frame, OpCode},
};
use core::ops::Range;

async fn recv<A, DRSR>(
  frame: Frame<&mut [u8], true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
) -> crate::Result<Range<usize>> {
  pkgs_aux.byte_buffer.clear();
  if let OpCode::Close = frame.op_code() {
    return Err(ClientApiFrameworkError::ClosedWsConnection.into());
  }
  pkgs_aux.byte_buffer.extend_from_copyable_slice(frame.payload())?;
  Ok(0..pkgs_aux.byte_buffer.len())
}

async fn send<A, DRSR, P, T>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, WsParams>,
  trans: &mut T,
  mut cb: impl for<'any> FnMutFut<
    (Frame<&'any mut Vector<u8>, true>, &'any mut T),
    Result = crate::Result<()>,
  >,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, WsParams>,
  T: Transport<DRSR, Params = WsParams>,
{
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *trans).await?;
  let op_code = match pkgs_aux.tp.ext_req_params_mut().ty {
    WsReqParamsTy::Bytes => OpCode::Binary,
    WsReqParamsTy::String => OpCode::Text,
  };
  cb.call((Frame::new_fin(op_code, &mut pkgs_aux.byte_buffer), trans)).await?;
  pkgs_aux.byte_buffer.clear();
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
