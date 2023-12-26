use crate::{
  client_api_framework::{
    dnsn::Deserialize,
    misc::log_res,
    network::transport::Transport,
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::AsyncBounds,
};
use core::{future::Future, ops::Range};

/// Bidirectional Transport
///
/// Similar to [Transport] but expects an connection where clients call poll data from the server.
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait BiTransport<DRSR>: Transport<DRSR> {
  /// Retrieves data from the server filling the internal buffer and returning the amount of
  /// bytes written.
  fn retrieve<A>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl AsyncBounds + Future<Output = crate::Result<Range<usize>>>
  where
    A: Api,
    A: AsyncBounds;

  /// Internally calls [Self::retrieve] and then tries to decode the defined response specified
  /// in [Package::ExternalResponseContent].
  #[inline]
  fn retrieve_and_decode_contained<A, P>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl AsyncBounds + Future<Output = Result<P::ExternalResponseContent, A::Error>>
  where
    A: Api,
    DRSR: AsyncBounds,
    P: Package<A, DRSR, Self::Params>,
    Self: AsyncBounds,
    Self::Params: AsyncBounds,
  {
    async {
      let range = self.retrieve(pkgs_aux).await?;
      log_res(pkgs_aux.byte_buffer.as_ref());
      let rslt = P::ExternalResponseContent::from_bytes(
        pkgs_aux.byte_buffer.get(range).unwrap_or_default(),
        &mut pkgs_aux.drsr,
      )?;
      pkgs_aux.byte_buffer.clear();
      Ok(rslt)
    }
  }
}

impl<DRSR, T> BiTransport<DRSR> for &mut T
where
  DRSR: AsyncBounds,
  T: AsyncBounds + BiTransport<DRSR>,
  T::Params: AsyncBounds,
  Self: AsyncBounds,
{
  #[inline]
  async fn retrieve<A>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> crate::Result<Range<usize>>
  where
    A: Api,
  {
    (**self).retrieve(pkgs_aux).await
  }
}
