use crate::{
  client_api_framework::{Api, pkg::Package},
  collection::Vector,
  de::{Id, protocol::JsonRpcEncoder},
};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

/// Used to store any type of helper data along side a package.
///
/// # Types
///
/// * `H`: Helper
/// * `P`: Package
#[derive(Clone, Debug)]
pub struct PkgWithHelper<H, P> {
  /// Helper
  pub helper: H,
  /// Package
  pub pkg: P,
}

impl<H, P> PkgWithHelper<H, P> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(helper: H, pkg: P) -> Self {
    Self { helper, pkg }
  }
}

impl<A, DRSR, H, P, T, TP> Package<A, DRSR, T, TP> for PkgWithHelper<H, P>
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
    self.pkg.after_sending((api, bytes, drsr), (trans, trans_params)).await
  }

  #[inline]
  async fn before_sending(
    &mut self,
    (api, bytes, drsr): (&mut A, &mut Vector<u8>, &mut DRSR),
    (trans, trans_params): (&mut T, &mut TP),
  ) -> Result<(), A::Error> {
    self.pkg.before_sending((api, bytes, drsr), (trans, trans_params)).await
  }

  #[inline]
  fn ext_req_content(&self) -> &Self::ExternalRequestContent {
    self.pkg.ext_req_content()
  }

  #[inline]
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
    self.pkg.ext_req_content_mut()
  }

  #[inline]
  fn pkg_params(&self) -> &Self::PackageParams {
    self.pkg.pkg_params()
  }

  #[inline]
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
    self.pkg.pkg_params_mut()
  }
}

impl<H, RP> Borrow<Id> for PkgWithHelper<H, JsonRpcEncoder<RP>> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.pkg.id
  }
}

impl<H, P> Eq for PkgWithHelper<H, P> where P: Eq {}

impl<H, P> Hash for PkgWithHelper<H, P>
where
  P: Hash,
{
  #[inline]
  fn hash<HA>(&self, state: &mut HA)
  where
    HA: Hasher,
  {
    self.pkg.hash(state);
  }
}

impl<H, P> Ord for PkgWithHelper<H, P>
where
  P: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.pkg.cmp(&other.pkg)
  }
}

impl<H, P> PartialEq for PkgWithHelper<H, P>
where
  P: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.pkg == other.pkg
  }
}

impl<H, P> PartialOrd for PkgWithHelper<H, P>
where
  P: PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.pkg.partial_cmp(&other.pkg)
  }
}
