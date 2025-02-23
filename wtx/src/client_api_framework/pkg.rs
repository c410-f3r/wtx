//! # Package
//!
//! Groups all elements that interact with packages.

mod batch_pkg;
mod pkg_with_helper;
mod pkgs_aux;

use crate::{
  client_api_framework::Api,
  data_transformation::dnsn::De,
  misc::{DecodeSeq, Encode, Vector},
};
pub use batch_pkg::{BatchElems, BatchPkg};
use core::future::Future;
pub use pkg_with_helper::*;
pub use pkgs_aux::*;

/// Groups all necessary information to define requests and responses as well as any desired
/// custom parameter to perform modifications before or after sending.
///
/// # Types
///
/// `A`: Associated API.
/// `DRSR`: DeserializeR/SerializeR
/// `T`: Transport
pub trait Package<A, DRSR, T, TP>
where
  A: Api,
{
  /// The expected data format that is going to be sent to an external actor.
  type ExternalRequestContent: Encode<De<DRSR>>;
  /// The expected data format returned by an external actor.
  type ExternalResponseContent<'de>: DecodeSeq<'de, De<DRSR>>;
  /// Any additional parameters used by this package.
  type PackageParams;

  /// Fallible hook that is automatically called after sending the request described in this
  /// package.
  #[inline]
  fn after_sending(
    &mut self,
    _: (&mut A, &mut Vector<u8>, &mut DRSR),
    _: (&mut T, &mut TP),
  ) -> impl Future<Output = Result<(), A::Error>> {
    async { Ok(()) }
  }

  /// Fallible hook that is automatically called before sending the request described in this
  /// package.
  #[inline]
  fn before_sending(
    &mut self,
    _: (&mut A, &mut Vector<u8>, &mut DRSR),
    _: (&mut T, &mut TP),
  ) -> impl Future<Output = Result<(), A::Error>> {
    async { Ok(()) }
  }

  /// External Request Content
  ///
  /// Instance value of the defined [`Self::ExternalRequestContent`].
  fn ext_req_content(&self) -> &Self::ExternalRequestContent;

  /// Similar to [`Self::ext_req_content`] but returns a mutable reference instead.
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent;

  /// Package Parameters
  ///
  /// Instance value of the defined [`Self::ExternalRequestContent`].
  fn pkg_params(&self) -> &Self::PackageParams;

  /// Similar to [`Self::pkg_params`] but returns a mutable reference instead.
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams;
}

impl<A, DRSR, T, TP> Package<A, DRSR, T, TP> for ()
where
  A: Api,
{
  type ExternalRequestContent = ();
  type ExternalResponseContent<'de> = ();
  type PackageParams = ();

  #[inline]
  fn ext_req_content(&self) -> &Self::ExternalRequestContent {
    &()
  }

  #[inline]
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
    self
  }

  #[inline]
  fn pkg_params(&self) -> &Self::PackageParams {
    self
  }

  #[inline]
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
    self
  }
}

impl<A, DRSR, P, T, TP> Package<A, DRSR, T, TP> for &mut P
where
  A: Api,
  P: Package<A, DRSR, T, TP>,
{
  type ExternalRequestContent = P::ExternalRequestContent;
  type ExternalResponseContent<'de> = P::ExternalResponseContent<'de>;
  type PackageParams = P::PackageParams;

  #[inline]
  async fn after_sending(
    &mut self,
    (api, bytes, drsr): (&mut A, &mut Vector<u8>, &mut DRSR),
    (trans, trans_params): (&mut T, &mut TP),
  ) -> Result<(), A::Error> {
    (**self).after_sending((api, bytes, drsr), (trans, trans_params)).await
  }

  #[inline]
  async fn before_sending(
    &mut self,
    (api, bytes, drsr): (&mut A, &mut Vector<u8>, &mut DRSR),
    (trans, trans_params): (&mut T, &mut TP),
  ) -> Result<(), A::Error> {
    (**self).before_sending((api, bytes, drsr), (trans, trans_params)).await
  }

  #[inline]
  fn ext_req_content(&self) -> &Self::ExternalRequestContent {
    (**self).ext_req_content()
  }

  #[inline]
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
    (**self).ext_req_content_mut()
  }

  #[inline]
  fn pkg_params(&self) -> &Self::PackageParams {
    (**self).pkg_params()
  }

  #[inline]
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
    (**self).pkg_params_mut()
  }
}
