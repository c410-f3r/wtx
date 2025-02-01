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
pub trait SendingTransport<TP>: Transport<TP> {
  /// Sends a request without trying to retrieve any counterpart data.
  fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>;
}

impl<T, TP> SendingTransport<TP> for &mut T
where
  T: SendingTransport<TP>,
{
  #[inline]
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    (**self).send(pkg, pkgs_aux).await
  }
}
