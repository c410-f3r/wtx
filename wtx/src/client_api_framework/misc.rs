//! Utility functions and structures

pub(crate) mod seq_visitor;

mod from_bytes;
mod generic_time;
mod pair;
mod query_writer;
mod request_counter;
mod request_limit;
mod request_throttling;
mod url;

use crate::client_api_framework::{
  dnsn::Serialize,
  network::transport::{Transport, TransportParams},
  pkg::{Package, PkgsAux},
  Api,
};
use core::{any::type_name, time::Duration};
pub use from_bytes::FromBytes;
pub use generic_time::*;
pub use pair::{Pair, PairMut};
pub use query_writer::QueryWriter;
pub use request_counter::RequestCounter;
pub use request_limit::RequestLimit;
pub use request_throttling::RequestThrottling;
pub use url::{Url, UrlString};

/// Useful when a request returns an optional field but the actual usage is within a
/// [core::result::Result] context.
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Sleeps for the specified amount of time.
///
/// Intended for asynchronous usage, i.e., won't block threads.
#[allow(
  // Depends on the selected set of features.
  clippy::unused_async
)]
#[inline]
pub async fn sleep(duration: Duration) -> crate::Result<()> {
  #[cfg(all(feature = "async-std", not(feature = "tokio")))]
  {
    async_std::task::sleep(duration).await;
    Ok(())
  }
  #[cfg(all(feature = "tokio", not(feature = "async-std")))]
  {
    tokio::time::sleep(duration).await;
    Ok(())
  }
  #[cfg(any(
    all(feature = "async-std", feature = "tokio"),
    all(not(feature = "tokio"), not(feature = "async-std"))
  ))]
  {
    // Open to better alternatives
    let now = GenericTime::now()?;
    loop {
      if now.elapsed()? >= duration {
        return Ok(());
      }
    }
  }
}

/// Used in all implementations of [crate::Transport::send] and/or
/// [crate::Transport::send_and_receive`].
#[allow(
  // Borrow checker woes
  clippy::needless_pass_by_value,
)]
pub(crate) fn log_req<DRSR, P, T>(
  _pgk: &mut P,
  _pkgs_aux: &mut PkgsAux<P::Api, DRSR, T::Params>,
  _trans: T,
) where
  P: Package<DRSR, T::Params>,
  T: Transport<DRSR>,
{
  _debug!(trans_ty = display(_trans.ty()), "Request: {:?}", {
    use crate::client_api_framework::dnsn::Serialize;
    let mut vec = alloc::vec::Vec::new();
    _pgk
      .ext_req_content_mut()
      .to_bytes(&mut vec, &mut _pkgs_aux.drsr)
      .and_then(|_| Ok(crate::misc::_from_utf8_basic_rslt(&vec)?.to_string()))
  });
}

/// Used in [crate::network::transport::Transport::send_retrieve_and_decode_contained] and all implementations of
/// [crate::Requests::decode_responses].
///
/// Not used in [crate::network::transport::Transport::send_retrieve_and_decode_batch] because
/// [crate::Requests::decode_responses] takes precedence.
pub(crate) fn log_res(_res: &[u8]) {
  _debug!("Response: {:?}", core::str::from_utf8(_res));
}

pub(crate) async fn manage_after_sending_related<DRSR, P, TP>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<P::Api, DRSR, TP>,
) -> Result<(), P::Error>
where
  P: Package<DRSR, TP>,
  TP: TransportParams,
{
  pkgs_aux.api.after_sending().await?;
  pkg.after_sending(&mut pkgs_aux.api, pkgs_aux.tp.ext_res_params_mut()).await?;
  Ok(())
}

pub(crate) async fn manage_before_sending_related<DRSR, P, T>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<P::Api, DRSR, T::Params>,
  trans: T,
) -> Result<(), P::Error>
where
  P: Package<DRSR, T::Params>,
  T: Transport<DRSR>,
{
  log_req(pkg, pkgs_aux, trans);
  pkg.ext_req_content_mut().to_bytes(&mut pkgs_aux.byte_buffer, &mut pkgs_aux.drsr)?;
  pkgs_aux.api.before_sending().await?;
  pkg
    .before_sending(&mut pkgs_aux.api, pkgs_aux.tp.ext_req_params_mut(), &pkgs_aux.byte_buffer)
    .await?;
  Ok(())
}
