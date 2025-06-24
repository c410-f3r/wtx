use crate::{
  client_api_framework::{
    Api,
    network::transport::Transport,
    pkg::{Package, PkgsAux},
  },
  de::{Decode, format::DecodeWrapper},
};

/// Transport that receives package data.
pub trait ReceivingTransport<TP>: Sized + Transport<TP> {
  /// Retrieves data from the server filling the internal buffer and returning the amount of
  /// bytes written.
  fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    req_id: Self::ReqId,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api;

  /// Internally calls [`Self::recv`] and then tries to decode the defined response specified
  /// in [`Package::ExternalResponseContent`].
  #[inline]
  fn recv_decode_contained<'de, A, DRSR, P>(
    &mut self,
    pkgs_aux: &'de mut PkgsAux<A, DRSR, TP>,
    req_id: Self::ReqId,
  ) -> impl Future<Output = Result<P::ExternalResponseContent<'de>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    async {
      self.recv(pkgs_aux, req_id).await?;
      Ok(P::ExternalResponseContent::decode(
        &mut pkgs_aux.drsr,
        &mut DecodeWrapper::new(&pkgs_aux.byte_buffer),
      )?)
    }
  }
}

impl<T, TP> ReceivingTransport<TP> for &mut T
where
  T: ReceivingTransport<TP>,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    req_id: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    (**self).recv(pkgs_aux, req_id).await
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::client_api_framework::{Api, network::transport::ReceivingTransport, pkg::PkgsAux};
  use tokio::sync::MappedMutexGuard;

  impl<T, TP> ReceivingTransport<TP> for MappedMutexGuard<'_, T>
  where
    T: ReceivingTransport<TP>,
  {
    #[inline]
    async fn recv<A, DRSR>(
      &mut self,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
      req_id: Self::ReqId,
    ) -> Result<(), A::Error>
    where
      A: Api,
    {
      (**self).recv(pkgs_aux, req_id).await
    }
  }
}
