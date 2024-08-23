use crate::{
  client_api_framework::{network::transport::TransportParams, pkg::Package, Api},
  data_transformation::dnsn::Serialize,
};
use core::marker::PhantomData;

/// Used to perform batch requests with multiple packages.
#[derive(Debug)]
pub struct BatchPkg<'slice, A, DRSR, P, TP>(BatchElems<'slice, A, DRSR, P, TP>, ());

impl<'slice, A, DRSR, P, TP> BatchPkg<'slice, A, DRSR, P, TP> {
  /// Currently, only slices of packages are allowed to perform batch requests.
  #[inline]
  pub fn new(slice: &'slice mut [P]) -> Self {
    Self(BatchElems(slice, PhantomData), ())
  }
}

impl<'slice, A, DRSR, P, TP> Package<A, DRSR, TP> for BatchPkg<'slice, A, DRSR, P, TP>
where
  A: Api,
  BatchElems<'slice, A, DRSR, P, TP>: Serialize<DRSR>,
  P: Package<A, DRSR, TP>,
  TP: TransportParams,
{
  type ExternalRequestContent = BatchElems<'slice, A, DRSR, P, TP>;
  type ExternalResponseContent<'de> = ();
  type PackageParams = ();

  #[inline]
  async fn after_sending(
    &mut self,
    api: &mut A,
    ext_res_params: &mut TP::ExternalResponseParams,
  ) -> Result<(), A::Error> {
    for elem in &mut *self.0 .0 {
      elem.after_sending(api, ext_res_params).await?;
    }
    Ok(())
  }

  #[inline]
  async fn before_sending(
    &mut self,
    api: &mut A,
    ext_req_params: &mut TP::ExternalRequestParams,
    req_bytes: &[u8],
  ) -> Result<(), A::Error> {
    for elem in &mut *self.0 .0 {
      elem.before_sending(api, ext_req_params, req_bytes).await?;
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
pub struct BatchElems<'slice, A, DRSR, P, T>(&'slice mut [P], PhantomData<(A, DRSR, T)>);

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    client_api_framework::{
      network::transport::TransportParams,
      pkg::{BatchElems, Package},
      Api,
    },
    data_transformation::dnsn::SerdeJson,
    misc::Vector,
  };
  use serde::Serializer;

  impl<A, DRSR, P, TP> crate::data_transformation::dnsn::Serialize<SerdeJson>
    for BatchElems<'_, A, DRSR, P, TP>
  where
    A: Api,
    P: Package<A, DRSR, TP>,
    P::ExternalRequestContent: serde::Serialize,
    TP: TransportParams,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      serde_json::Serializer::new(bytes)
        .collect_seq(self.0.iter().map(Package::ext_req_content))?;
      Ok(())
    }
  }
}
