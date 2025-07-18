use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    network::transport::{ReceivingTransport, SendingTransport},
    pkg::{BatchElems, BatchPkg, Package, PkgsAux},
  },
  collection::Vector,
  de::{
    Decode, DecodeSeq, Encode,
    format::{De, DecodeWrapper},
  },
};

/// Transport that sends and receives package data
///
/// # Types
///
/// * `DRSR`: `D`eserialize`R`/`S`erialize`R`
pub trait SendingReceivingTransport<TP>: ReceivingTransport<TP> + SendingTransport<TP> {
  /// Sends a sequence of bytes and then awaits its counterpart data response.
  ///
  /// The returned bytes are stored in `pkgs_aux` and its length is returned by this method.
  #[inline]
  fn send_bytes_recv<A, DRSR>(
    &mut self,
    bytes: SendBytesSource<'_>,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
  {
    async move {
      let req_id = self.send_bytes(bytes, pkgs_aux).await?;
      self.recv(pkgs_aux, req_id).await
    }
  }

  /// Sends a package and then awaits its counterpart data response.
  ///
  /// The returned bytes are stored in `pkgs_aux` and its length is returned by this method.
  #[inline]
  fn send_pkg_recv<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    async {
      let req_id = self.send_pkg(pkg, pkgs_aux).await?;
      self.recv(pkgs_aux, req_id).await
    }
  }

  /// Convenient method similar to [`Self::send_pkg_recv_decode_contained`] but used for batch
  /// requests.
  ///
  /// All the expected data must be available in a single response.
  #[inline]
  fn send_pkg_recv_decode_batch<'pkgs, 'pkgs_aux, A, DRSR, P>(
    &mut self,
    buffer: &mut Vector<P::ExternalResponseContent<'pkgs_aux>>,
    pkgs: &'pkgs mut [P],
    pkgs_aux: &'pkgs_aux mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<(), A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
    BatchElems<'pkgs, A, DRSR, P, Self::Inner, TP>: Encode<De<DRSR>>,
  {
    async {
      self.send_pkg_recv(&mut BatchPkg::new(pkgs, pkgs_aux), pkgs_aux).await?;
      P::ExternalResponseContent::decode_seq(
        &mut pkgs_aux.drsr,
        buffer,
        &mut DecodeWrapper::new(&pkgs_aux.byte_buffer),
      )?;
      Ok(())
    }
  }

  /// Internally calls [`Self::send_pkg_recv`] and then tries to decode the defined response specified
  /// in [`Package::ExternalResponseContent`].
  #[inline]
  fn send_pkg_recv_decode_contained<'de, A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &'de mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<P::ExternalResponseContent<'de>, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    async {
      self.send_pkg_recv(pkg, pkgs_aux).await?;
      Ok(P::ExternalResponseContent::decode(
        &mut pkgs_aux.drsr,
        &mut DecodeWrapper::new(&pkgs_aux.byte_buffer),
      )?)
    }
  }
}

impl<T, TP> SendingReceivingTransport<TP> for T where
  T: ReceivingTransport<TP> + SendingTransport<TP>
{
}
