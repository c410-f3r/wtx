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
  misc::{UriString, from_utf8_basic},
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

pub(crate) fn log_req<T, TP>(
  _bytes: &[u8],
  _log_body: bool,
  _trans: &mut T,
  _uri: Option<&UriString>,
) where
  T: Transport<TP>,
{
  let _body = if _log_body { from_utf8_basic(_bytes).ok() } else { None };
  _debug!(trans_ty = display(_trans.ty()), r#"Request ({_uri:?}) "{_body:?}"#);
}
