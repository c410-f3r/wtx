use crate::{
  client_api_framework::{
    misc::log_res,
    network::transport::{RecievingTransport, SendingTransport},
    pkg::{BatchElems, BatchPkg, Package, PkgsAux},
    Api,
  },
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::{Lease, Vector},
};
use core::{future::Future, ops::Range};

/// Transport that sends and receives package data
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait SendingRecievingTransport<DRSR>:
  RecievingTransport<DRSR> + SendingTransport<DRSR>
{
  /// Sends a request and then awaits its counterpart data response.
  ///
  /// The returned bytes are stored in `pkgs_aux` and its length is returned by this method.
  #[inline]
  fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<Range<usize>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    async {
      self.send(pkg, pkgs_aux).await?;
      self.recv(pkgs_aux).await
    }
  }

  /// Convenient method similar to [`Self::send_recv_decode_contained`] but used for batch
  /// requests.
  ///
  /// All the expected data must be available in a single response.
  #[inline]
  fn send_recv_decode_batch<'pkgs, 'pkgs_aux, A, P>(
    &mut self,
    buffer: &mut Vector<P::ExternalResponseContent<'pkgs_aux>>,
    pkgs: &'pkgs mut [P],
    pkgs_aux: &'pkgs_aux mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
    BatchElems<'pkgs, A, DRSR, P, Self::Params>: Serialize<DRSR>,
  {
    async {
      let range = self.send_recv(&mut BatchPkg::new(pkgs), pkgs_aux).await?;
      log_res(pkgs_aux.byte_buffer.lease());
      P::ExternalResponseContent::seq_from_bytes(
        buffer,
        pkgs_aux.byte_buffer.get(range).unwrap_or_default(),
        &mut pkgs_aux.drsr,
      )?;
      Ok(())
    }
  }

  /// Internally calls [`Self::send_recv`] and then tries to decode the defined response specified
  /// in [`Package::ExternalResponseContent`].
  #[inline]
  fn send_recv_decode_contained<'de, A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &'de mut PkgsAux<A, DRSR, Self::Params>,
  ) -> impl Future<Output = Result<P::ExternalResponseContent<'de>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Params>,
  {
    async {
      let range = self.send_recv(pkg, pkgs_aux).await?;
      log_res(pkgs_aux.byte_buffer.lease());
      Ok(P::ExternalResponseContent::from_bytes(
        pkgs_aux.byte_buffer.get(range).unwrap_or_default(),
        &mut pkgs_aux.drsr,
      )?)
    }
  }
}

impl<DRSR, T> SendingRecievingTransport<DRSR> for T where
  T: RecievingTransport<DRSR> + SendingTransport<DRSR>
{
}
