use crate::client_api_framework::{
  network::transport::Transport,
  pkg::{Package, PkgsAux},
  Api,
};
use core::future::Future;

/// Transport that sends package data.
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait SendingTransport<DRSR>: Transport<DRSR> {
  /// Sends a request without trying to retrieve any counterpart data.
  fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>;
}

impl<DRSR, T> SendingTransport<DRSR> for &mut T
where
  T: SendingTransport<DRSR>,
{
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
    (**self).send(pkg, pkgs_aux).await
  }
}
