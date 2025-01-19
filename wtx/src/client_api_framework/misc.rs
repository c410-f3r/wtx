//! Utility functions and structures

mod from_bytes;
mod pair;
mod request_counter;
mod request_limit;
mod request_throttling;

use crate::{
  client_api_framework::{
    network::transport::{Transport, TransportParams},
    pkg::{Package, PkgsAux},
    Api,
  },
  data_transformation::dnsn::Serialize,
};
pub use from_bytes::FromBytes;
pub use pair::{Pair, PairMut};
pub use request_counter::RequestCounter;
pub use request_limit::RequestLimit;
pub use request_throttling::RequestThrottling;

/// Used in all implementations of [`crate::Transport::send`] and/or
/// [`crate::Transport::send_recv``].
#[inline]
pub(crate) fn log_req<A, DRSR, P, T>(
  _pgk: &mut P,
  _pkgs_aux: &mut PkgsAux<A, DRSR, T::Params>,
  _trans: &mut T,
) where
  A: Api,
  P: Package<A, DRSR, T::Params>,
  T: Transport,
{
  _debug!(trans_ty = display(_trans.ty()), "Request: {:?}", {
    use crate::data_transformation::dnsn::Serialize;
    let mut vec = crate::misc::Vector::new();
    _pgk
      .ext_req_content_mut()
      .to_bytes(&mut vec, &mut _pkgs_aux.drsr)
      .and_then(|_| Ok(alloc::string::String::from(crate::misc::from_utf8_basic(&vec)?)))
  });
}

/// Used in [`crate::network::transport::Transport::send_recv_decode_contained`] and all implementations of
/// [`crate::Requests::decode_responses`].
///
/// Not used in [`crate::network::transport::Transport::send_recv_decode_batch`] because
/// [`crate::Requests::decode_responses`] takes precedence.
#[inline]
pub(crate) fn log_res(_res: &[u8]) {
  _debug!("Response: {:?}", crate::misc::from_utf8_basic(_res));
}

#[inline]
pub(crate) async fn manage_after_sending_related<A, DRSR, P, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, TP>,
  TP: TransportParams,
{
  pkgs_aux.api.after_sending().await?;
  pkg.after_sending(&mut pkgs_aux.api, pkgs_aux.tp.ext_res_params_mut()).await?;
  Ok(())
}

#[inline]
pub(crate) async fn manage_before_sending_related<A, DRSR, P, T>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, T::Params>,
  trans: &mut T,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T::Params>,
  T: Transport,
{
  log_req(pkg, pkgs_aux, trans);
  pkg.ext_req_content_mut().to_bytes(&mut pkgs_aux.byte_buffer, &mut pkgs_aux.drsr)?;
  pkgs_aux.api.before_sending().await?;
  pkg
    .before_sending(&mut pkgs_aux.api, pkgs_aux.tp.ext_req_params_mut(), &pkgs_aux.byte_buffer)
    .await?;
  Ok(())
}
