//! # Package
//!
//! Groups all elements that interact with packages.

mod batch_pkg;
mod pkg_with_helper;
mod pkgs_aux;

use crate::client_api_framework::{
  dnsn::{Deserialize, Serialize},
  network::transport::TransportParams,
  Api,
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
/// `TP`: Transport Parameters
pub trait Package<A, DRSR, TP>
where
  A: Api,
  TP: TransportParams,
{
  /// The expected data format that is going to be sent to an external actor.
  type ExternalRequestContent: Serialize<DRSR>;
  /// The expected data format returned by an external actor.
  type ExternalResponseContent: Deserialize<DRSR>;
  /// Any additional parameters used by this package.
  type PackageParams;

  /// Fallible hook that is automatically called after sending the request described in this
  /// package.
  #[inline]
  fn after_sending(
    &mut self,
    _: &mut A,
    _: &mut TP::ExternalResponseParams,
  ) -> impl Future<Output = Result<(), A::Error>> {
    async { Ok(()) }
  }

  /// Fallible hook that is automatically called before sending the request described in this
  /// package.
  #[inline]
  fn before_sending(
    &mut self,
    _: &mut A,
    _: &mut TP::ExternalRequestParams,
    _: &[u8],
  ) -> impl Future<Output = Result<(), A::Error>> {
    async { Ok(()) }
  }

  /// External Request Content
  ///
  /// Instance value of the defined [Self::ExternalRequestContent].
  fn ext_req_content(&self) -> &Self::ExternalRequestContent;

  /// Similar to [Self::ext_req_content] but returns a mutable reference instead.
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent;

  /// Package Parameters
  ///
  /// Instance value of the defined [Self::ExternalRequestContent].
  fn pkg_params(&self) -> &Self::PackageParams;

  /// Similar to [Self::pkg_params] but returns a mutable reference instead.
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams;
}

impl<DRSR, TP> Package<(), DRSR, TP> for ()
where
  TP: TransportParams,
{
  type ExternalRequestContent = ();
  type ExternalResponseContent = ();
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

impl<A, DRSR, P, TP> Package<A, DRSR, TP> for &mut P
where
  A: Api,
  P: Package<A, DRSR, TP>,
  TP: TransportParams,
{
  type ExternalRequestContent = P::ExternalRequestContent;
  type ExternalResponseContent = P::ExternalResponseContent;
  type PackageParams = P::PackageParams;

  #[inline]
  async fn after_sending(
    &mut self,
    api: &mut A,
    ext_res_params: &mut TP::ExternalResponseParams,
  ) -> Result<(), A::Error> {
    (**self).after_sending(api, ext_res_params).await
  }

  #[inline]
  async fn before_sending(
    &mut self,
    api: &mut A,
    ext_req_params: &mut TP::ExternalRequestParams,
    req_bytes: &[u8],
  ) -> Result<(), A::Error> {
    (**self).before_sending(api, ext_req_params, req_bytes).await
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
