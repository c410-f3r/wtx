//! Utility functions and structures

mod from_bytes;
mod pair;
mod request_counter;
mod request_limit;
mod request_throttling;

use crate::{
  client_api_framework::{
    Api,
    network::transport::Transport,
    pkg::{Package, PkgsAux},
  },
  de::{Encode, format::EncodeWrapper},
  misc::from_utf8_basic,
};
pub use from_bytes::FromBytes;
pub use pair::{Pair, PairMut};
pub use request_counter::RequestCounter;
pub use request_limit::RequestLimit;
pub use request_throttling::RequestThrottling;

pub(crate) async fn manage_after_sending_bytes<A, DRSR, TP>(
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
{
  pkgs_aux.api.after_sending().await?;
  Ok(())
}

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

pub(crate) async fn manage_before_sending_bytes<A, DRSR, TP>(
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
{
  pkgs_aux.api.before_sending().await?;
  Ok(())
}

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
  Ok(())
}

#[cfg(feature = "http")]
pub(crate) fn log_http_req<T, TP>(
  _bytes: &[u8],
  _log_body: bool,
  method: crate::http::Method,
  _trans: &T,
  _uri: &crate::misc::UriString,
) where
  T: Transport<TP>,
{
  let _body = if _log_body { from_utf8_basic(_bytes).ok() } else { None };
  _debug!(
    body = display(_body.unwrap_or_default()),
    method = %method,
    trans_ty = display(_trans.ty()),
    uri = display(_uri.as_str()),
    "Request"
  );
}

pub(crate) fn log_req<T, TP>(_bytes: &[u8], _log_body: bool, _trans: &T)
where
  T: Transport<TP>,
{
  let _body = if _log_body { from_utf8_basic(_bytes).ok() } else { None };
  _debug!(body = display(_body.unwrap_or_default()), trans_ty = display(_trans.ty()), "Request");
}
