use crate::client_api_framework::{
  Api,
  network::transport::Transport,
  pkg::{Package, PkgsAux},
};

/// Transport that sends package data.
pub trait SendingTransport<TP>: Transport<TP> {
  /// Sends a sequence of bytes without trying to retrieve any counterpart data.
  ///
  /// If `bytes` is `None`, then the buffer of `pkgs_aux` will be sent.
  fn send_bytes<A, DRSR>(
    &mut self,
    bytes: Option<&[u8]>,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<Self::ReqId, A::Error>>
  where
    A: Api;

  /// Sends a package without trying to retrieve any counterpart data.
  fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> impl Future<Output = Result<Self::ReqId, A::Error>>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>;
}

impl<T, TP> SendingTransport<TP> for &mut T
where
  T: SendingTransport<TP>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: Option<&[u8]>,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Self::ReqId, A::Error>
  where
    A: Api,
  {
    (**self).send_bytes(bytes, pkgs_aux).await
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Self::ReqId, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    (**self).send_pkg(pkg, pkgs_aux).await
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::client_api_framework::{
    Api,
    network::transport::SendingTransport,
    pkg::{Package, PkgsAux},
  };
  use tokio::sync::MappedMutexGuard;

  impl<T, TP> SendingTransport<TP> for MappedMutexGuard<'_, T>
  where
    T: SendingTransport<TP>,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: Option<&[u8]>,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      (**self).send_bytes(bytes, pkgs_aux).await
    }

    #[inline]
    async fn send_pkg<A, DRSR, P>(
      &mut self,
      pkg: &mut P,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
      P: Package<A, DRSR, Self::Inner, TP>,
    {
      (**self).send_pkg(pkg, pkgs_aux).await
    }
  }
}
