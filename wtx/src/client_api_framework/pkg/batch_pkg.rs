use crate::{
  client_api_framework::{
    network::transport::TransportParams, pkg::Package, Api, ClientApiFrameworkError,
  },
  data_transformation::{
    dnsn::{Deserialize, Serialize},
    Id,
  },
  misc::Lease,
};
use cl_aux::DynContigColl;
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

impl<A, DRSR, P, TP> BatchPkg<'_, A, DRSR, P, TP>
where
  A: Api,
  P: Package<A, DRSR, TP>,
  P::ExternalRequestContent: Lease<Id> + Ord,
  for<'de> P::ExternalResponseContent<'de>: Lease<Id> + Ord,
  TP: TransportParams,
{
  /// Deserializes a sequence of bytes and then pushes them to the provided buffer.
  #[inline]
  pub fn decode_and_push_from_bytes<B, E>(
    &mut self,
    buffer: &mut B,
    bytes: &[u8],
    drsr: &mut DRSR,
  ) -> Result<(), A::Error>
  where
    A::Error: From<E>,
    B: for<'de> DynContigColl<E, P::ExternalResponseContent<'de>>,
  {
    if self.0 .0.is_empty() {
      return Ok(());
    }
    Self::is_sorted(self.0 .0.iter().map(|elem| elem.ext_req_content().lease()))?;
    let mut pkgs_idx = 0;
    let mut responses_are_not_sorted = false;
    P::ExternalResponseContent::seq_from_bytes(bytes, drsr, |eresc| {
      let eresc_id = *eresc.lease();
      let found_pkgs_idx = Self::search_slice(pkgs_idx, eresc_id, self.0 .0)?;
      if pkgs_idx != found_pkgs_idx {
        responses_are_not_sorted = true;
      }
      buffer.push(eresc).map_err(Into::into)?;
      pkgs_idx = pkgs_idx.wrapping_add(1);
      Ok::<_, A::Error>(())
    })?;
    if responses_are_not_sorted {
      buffer.sort_unstable();
    }
    Ok(())
  }

  fn is_sorted<T>(mut iter: impl Iterator<Item = T>) -> crate::Result<()>
  where
    T: PartialOrd,
  {
    let mut is_sorted = true;
    let Some(mut previous) = iter.next() else {
      return Ok(());
    };
    for curr in iter {
      if previous > curr {
        is_sorted = false;
        break;
      }
      previous = curr;
    }
    if is_sorted {
      Ok(())
    } else {
      Err(ClientApiFrameworkError::BatchPackagesAreNotSorted.into())
    }
  }

  // First try indexing and then falls back to binary search
  fn search_slice(idx: usize, eresc_id: Id, pkgs: &[P]) -> crate::Result<usize> {
    if pkgs.get(idx).map(|pkg| *pkg.ext_req_content().lease() == eresc_id).unwrap_or_default() {
      return Ok(idx);
    }
    pkgs.binary_search_by(|req| req.ext_req_content().lease().cmp(&&eresc_id)).ok().ok_or(
      ClientApiFrameworkError::ResponseIdIsNotPresentInTheOfSentBatchPackages(eresc_id).into(),
    )
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
