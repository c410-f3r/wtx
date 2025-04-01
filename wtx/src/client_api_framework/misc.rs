//! Utility functions and structures

mod from_bytes;
mod pair;
mod request_counter;
mod request_limit;
mod request_throttling;

use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    network::{TransportGroup, transport::Transport},
    pkg::{Package, PkgsAux},
  },
  data_transformation::dnsn::EncodeWrapper,
  misc::Encode,
};
pub use from_bytes::FromBytes;
pub use pair::{Pair, PairMut};
pub use request_counter::RequestCounter;
pub use request_limit::RequestLimit;
pub use request_throttling::RequestThrottling;

/// Used in [`crate::network::transport::Transport::send_recv_decode_contained`] and all implementations of
/// [`crate::Requests::decode_responses`].
///
/// Not used in [`crate::network::transport::Transport::send_recv_decode_batch`] because
/// [`crate::Requests::decode_responses`] takes precedence.
#[inline]
pub(crate) fn _log_res(_log_body: bool, _res: &[u8], _tg: TransportGroup) {
  if _log_body {
    _debug!(trans_ty = display(_tg), "Response: {:?}", crate::misc::from_utf8_basic(_res));
  } else {
    _debug!(trans_ty = display(_tg), "Response");
  }
}

#[inline]
pub(crate) async fn manage_after_sending_bytes<A, DRSR, TP>(
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
{
  pkgs_aux.api.after_sending().await?;
  Ok(())
}

#[inline]
pub(crate) async fn manage_after_sending_pkg<A, DRSR, P, T, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T, TP>,
  T: Transport<TP>,
{
  pkgs_aux.api.after_sending().await?;
  pkg
    .after_sending(
      (&mut pkgs_aux.api, &mut pkgs_aux.byte_buffer, &mut pkgs_aux.drsr),
      (trans, &mut pkgs_aux.tp),
    )
    .await?;
  Ok(())
}

#[inline]
pub(crate) async fn manage_before_sending_bytes<A, DRSR, T, TP>(
  bytes: SendBytesSource<'_>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
) -> Result<(), A::Error>
where
  A: Api,
  T: Transport<TP>,
{
  log_req(bytes.bytes(&pkgs_aux.byte_buffer), pkgs_aux.log_body.1, trans);
  pkgs_aux.api.before_sending().await?;
  Ok(())
}

#[inline]
pub(crate) async fn manage_before_sending_pkg<A, DRSR, P, T, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  trans: &mut T,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T, TP>,
  T: Transport<TP>,
{
  pkgs_aux.api.before_sending().await?;
  pkg
    .before_sending(
      (&mut pkgs_aux.api, &mut pkgs_aux.byte_buffer, &mut pkgs_aux.drsr),
      (trans, &mut pkgs_aux.tp),
    )
    .await?;
  pkg
    .ext_req_content_mut()
    .encode(&mut pkgs_aux.drsr, &mut EncodeWrapper::new(&mut pkgs_aux.byte_buffer))?;
  log_req(&pkgs_aux.byte_buffer, pkgs_aux.log_body.1, trans);
  Ok(())
}

#[inline]
fn log_req<T, TP>(_bytes: &[u8], _log_body: bool, _trans: &mut T)
where
  T: Transport<TP>,
{
  if _log_body {
    _debug!(trans_ty = display(_trans.ty()), "Request: {:?}", crate::misc::from_utf8_basic(_bytes));
  } else {
    _debug!(trans_ty = display(_trans.ty()), "Request");
  }
}
