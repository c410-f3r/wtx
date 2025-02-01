use crate::{
  client_api_framework::{pkg::Package, Api},
  data_transformation::dnsn::Serialize,
  misc::Vector,
};
use core::marker::PhantomData;

/// Used to perform batch requests with multiple packages.
#[derive(Debug)]
pub struct BatchPkg<'slice, A, DRSR, P, T, TP>(BatchElems<'slice, A, DRSR, P, T, TP>, ());

impl<'slice, A, DRSR, P, T, TP> BatchPkg<'slice, A, DRSR, P, T, TP> {
  /// Currently, only slices of packages are allowed to perform batch requests.
  #[inline]
  pub fn new(slice: &'slice mut [P]) -> Self {
    Self(BatchElems(slice, PhantomData), ())
  }
}

impl<'slice, A, DRSR, P, T, TP> Package<A, DRSR, T, TP> for BatchPkg<'slice, A, DRSR, P, T, TP>
where
  A: Api,
  BatchElems<'slice, A, DRSR, P, T, TP>: Serialize<DRSR>,
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
    for elem in &mut *self.0 .0 {
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
    for elem in &mut *self.0 .0 {
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
      network::transport::Transport,
      pkg::{BatchElems, Package},
      Api,
    },
    data_transformation::dnsn::SerdeJson,
    misc::Vector,
  };
  use serde::Serializer;

  impl<A, DRSR, P, T, TP> crate::data_transformation::dnsn::Serialize<SerdeJson>
    for BatchElems<'_, A, DRSR, P, T, TP>
  where
    A: Api,
    P: Package<A, DRSR, T, TP>,
    P::ExternalRequestContent: serde::Serialize,
    T: Transport<TP>,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      serde_json::Serializer::new(bytes)
        .collect_seq(self.0.iter().map(Package::ext_req_content))?;
      Ok(())
    }
  }
}
