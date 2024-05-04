//! Implementations of the [Transport] trait.

mod bi_transport;
mod mock;
#[cfg(feature = "std")]
mod std;
mod transport_params;
mod unit;
#[cfg(feature = "web-socket")]
mod wtx_ws;

use crate::client_api_framework::{
  dnsn::{Deserialize, Serialize},
  misc::log_res,
  network::TransportGroup,
  pkg::{BatchElems, BatchPkg, Package, PkgsAux},
  Api, Id,
};
pub use bi_transport::*;
use cl_aux::DynContigColl;
use core::{borrow::Borrow, future::Future, ops::Range};
pub use mock::*;
pub use transport_params::*;

/// Any means of transferring data between two parties.
///
/// Please, see the [crate::pkg::Package] implementation of the desired package to know
/// more about the expected types as well as any other additional documentation.
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait Transport<DRSR> {
  /// Every transport has an [TransportGroup] identifier.
  const GROUP: TransportGroup;
  /// Every transport has request and response parameters.
  type Params: TransportParams;

  /// Sends a request without trying to retrieve any counterpart data.
  fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>;

  /// Sends a request and then awaits its counterpart data response.
  ///
  /// The returned bytes are stored in `pkgs_aux` and its length is returned by this method.
  fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<Range<usize>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>;

  /// Convenient method similar to [Self::send_recv_decode_contained] but used for batch
  /// requests.
  ///
  /// All the expected data must be available in a single response.
  #[inline]
  fn send_recv_decode_batch<A, P, RESS>(
    &mut self,
    pkgs: &mut [P],
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
    ress: &mut RESS,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
    P::ExternalRequestContent: Borrow<Id> + Ord,
    P::ExternalResponseContent: Borrow<Id> + Ord,
    RESS: DynContigColl<P::ExternalResponseContent>,
    for<'any> BatchElems<'any, A, DRSR, P, Self::Params>: Serialize<DRSR>,
  {
    async {
      let batch_package = &mut BatchPkg::new(pkgs);
      let range = self.send_recv(batch_package, pkgs_aux).await?;
      log_res(pkgs_aux.byte_buffer.as_slice());
      batch_package.decode_and_push_from_bytes(
        ress,
        pkgs_aux.byte_buffer.get(range).unwrap_or_default(),
        &mut pkgs_aux.drsr,
      )?;
      Ok(())
    }
  }

  /// Internally calls [Self::send_recv] and then tries to decode the defined response specified
  /// in [Package::ExternalResponseContent].
  #[inline]
  fn send_recv_decode_contained<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<P::ExternalResponseContent, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    async {
      let range = self.send_recv(pkg, pkgs_aux).await?;
      log_res(pkgs_aux.byte_buffer.as_slice());
      Ok(P::ExternalResponseContent::from_bytes(
        pkgs_aux.byte_buffer.get(range).unwrap_or_default(),
        &mut pkgs_aux.drsr,
      )?)
    }
  }

  /// Instance counterpart of [Self::GROUP].
  #[inline]
  fn ty(&self) -> TransportGroup {
    Self::GROUP
  }
}

impl<DRSR, T> Transport<DRSR> for &mut T
where
  T: Transport<DRSR>,
{
  const GROUP: TransportGroup = T::GROUP;
  type Params = T::Params;

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

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    (**self).send_recv(pkg, pkgs_aux).await
  }
}

#[cfg(test)]
mod tests {
  use crate::client_api_framework::{
    dnsn::{Deserialize, Serialize},
    network::transport::TransportParams,
    pkg::Package,
  };
  use alloc::vec::Vec;

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct PingPong(pub(crate) Ping, pub(crate) ());

  impl<DRSR, TP> Package<(), DRSR, TP> for PingPong
  where
    TP: TransportParams,
  {
    type ExternalRequestContent = Ping;
    type ExternalResponseContent = Pong;
    type PackageParams = ();

    fn ext_req_content(&self) -> &Self::ExternalRequestContent {
      &self.0
    }

    fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
      &mut self.0
    }

    fn pkg_params(&self) -> &Self::PackageParams {
      &self.1
    }

    fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
      &mut self.1
    }
  }

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct Ping;

  impl<DRSR> Serialize<DRSR> for Ping {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut DRSR) -> crate::Result<()> {
      bytes.extend(*b"ping");
      Ok(())
    }
  }

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct Pong(pub(crate) &'static str);

  impl<DRSR> Deserialize<DRSR> for Pong {
    fn from_bytes(bytes: &[u8], _: &mut DRSR) -> crate::Result<Self> {
      assert_eq!(bytes, b"ping");
      Ok(Self("pong"))
    }

    fn seq_from_bytes<E>(
      _: &[u8],
      _: &mut DRSR,
      _: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: From<crate::Error>,
    {
      Ok(())
    }
  }
}
