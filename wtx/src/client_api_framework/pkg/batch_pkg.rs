use crate::{
  client_api_framework::{
    Api,
    pkg::{Package, PkgsAux},
  },
  collection::Vector,
  de::{Encode, format::De},
};
use core::marker::PhantomData;

/// Used to perform batch requests with multiple packages.
#[derive(Debug)]
pub struct BatchPkg<'slice, A, DRSR, P, T, TP>(BatchElems<'slice, A, DRSR, P, T, TP>, ());

impl<'slice, A, DRSR, P, T, TP> BatchPkg<'slice, A, DRSR, P, T, TP> {
  /// Currently, only slices of packages are allowed to perform batch requests.
  #[inline]
  pub const fn new(pkgs: &'slice mut [P], pkgs_aux: &mut PkgsAux<A, DRSR, TP>) -> Self {
    pkgs_aux.log_body = (false, pkgs_aux.log_body.0);
    Self(BatchElems(pkgs, PhantomData), ())
  }
}

impl<'slice, A, DRSR, P, T, TP> Package<A, DRSR, T, TP> for BatchPkg<'slice, A, DRSR, P, T, TP>
where
  A: Api,
  BatchElems<'slice, A, DRSR, P, T, TP>: Encode<De<DRSR>>,
  P: Package<A, DRSR, T, TP>,
{
  type ExternalRequestContent = BatchElems<'slice, A, DRSR, P, T, TP>;
  type ExternalResponseContent<'de> = ();
  type PackageParams = ();

  #[inline]
  async fn after_sending(
    &mut self,
    (api, bytes, drsr): (&mut A, &mut Vector<u8>, &mut DRSR),
    (trans, trans_params): (&mut T, &mut TP),
  ) -> Result<(), A::Error> {
    for elem in &mut *self.0.0 {
      elem.after_sending((api, bytes, drsr), (trans, trans_params)).await?;
    }
    Ok(())
  }

  #[inline]
  async fn before_sending(
    &mut self,
    (api, bytes, drsr): (&mut A, &mut Vector<u8>, &mut DRSR),
    (trans, trans_params): (&mut T, &mut TP),
  ) -> Result<(), A::Error> {
    for elem in &mut *self.0.0 {
      elem.before_sending((api, bytes, drsr), (trans, trans_params)).await?;
    }
    Ok(())
  }

  #[inline]
  fn ext_req_content(&self) -> &Self::ExternalRequestContent {
    &self.0
  }

  #[inline]
  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
    &mut self.0
  }

  #[inline]
  fn pkg_params(&self) -> &Self::PackageParams {
    &self.1
  }

  #[inline]
  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
    &mut self.1
  }
}

/// Used internally and exclusively by [BatchPkg]. Not intended for public usage.
#[derive(Debug)]
pub struct BatchElems<'slice, A, DRSR, P, T, TP>(&'slice mut [P], PhantomData<(A, DRSR, T, TP)>);

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    client_api_framework::{
      Api,
      network::transport::Transport,
      pkg::{BatchElems, Package},
    },
    de::{
      Encode,
      format::{De, EncodeWrapper, SerdeJson},
    },
  };
  use serde::Serializer as _;

  impl<A, DRSR, P, T, TP> Encode<De<SerdeJson>> for BatchElems<'_, A, DRSR, P, T, TP>
  where
    A: Api,
    P: Package<A, DRSR, T, TP>,
    P::ExternalRequestContent: serde::Serialize,
    T: Transport<TP>,
  {
    #[inline]
    fn encode(&self, _: &mut SerdeJson, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
      serde_json::Serializer::new(&mut *ew.vector)
        .collect_seq(self.0.iter().map(Package::ext_req_content))?;
      Ok(())
    }
  }
}
