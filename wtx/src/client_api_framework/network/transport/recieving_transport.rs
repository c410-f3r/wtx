use crate::{
  client_api_framework::{
    Api,
    network::transport::Transport,
    pkg::{Package, PkgsAux},
  },
  data_transformation::dnsn::DecodeWrapper,
  misc::Decode,
};
use core::{future::Future, ops::Range};

/// Transport that receives package data.
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait RecievingTransport<TP>: Sized + Transport<TP> {
  /// Retrieves data from the server filling the internal buffer and returning the amount of
  /// bytes written.
  fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<Range<usize>, A::Error>>
  where
    A: Api;

  /// Internally calls [`Self::retrieve`] and then tries to decode the defined response specified
  /// in [`Package::ExternalResponseContent`].
  #[inline]
  fn recv_decode_contained<'de, A, DRSR, P>(
    &mut self,
    pkgs_aux: &'de mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<P::ExternalResponseContent<'de>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    async {
      let range = self.recv(pkgs_aux).await?;
      Ok(P::ExternalResponseContent::decode(
        &mut pkgs_aux.drsr,
        &mut DecodeWrapper::new(pkgs_aux.byte_buffer.get(range).unwrap_or_default()),
      )?)
    }
  }
}

impl<T, TP> RecievingTransport<TP> for &mut T
where
  T: RecievingTransport<TP>,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    (**self).recv(pkgs_aux).await
  }
}
