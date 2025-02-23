use crate::client_api_framework::{
  Api,
  misc::{
    manage_after_sending_bytes, manage_after_sending_pkg, manage_before_sending_bytes,
    manage_before_sending_pkg,
  },
  network::{
    TransportGroup,
    transport::{RecievingTransport, SendingTransport, Transport},
  },
  pkg::{Package, PkgsAux},
};
use core::ops::Range;

impl<TP> RecievingTransport<TP> for () {
  #[inline]
  async fn recv<A, DRSR>(&mut self, _: &mut PkgsAux<A, DRSR, TP>) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..0)
  }
}

impl<TP> SendingTransport<TP> for () {
  #[inline]
  async fn send_bytes<A>(
    &mut self,
    bytes: &[u8],
    pkgs_aux: &mut PkgsAux<A, (), TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    manage_before_sending_bytes(bytes, pkgs_aux, self).await?;
    manage_after_sending_bytes(pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    manage_before_sending_pkg(pkg, pkgs_aux, self).await?;
    manage_after_sending_pkg(pkg, pkgs_aux, self).await?;
    Ok(())
  }
}

/// Does absolutely nothing. Good for demonstration purposes.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{network::transport::SendingReceivingTransport, pkg::PkgsAux};
/// let _ =
///   ().send_pkg_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ())).await?;
/// # Ok(()) }
/// ```
impl<TP> Transport<TP> for () {
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Inner = ();
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::client_api_framework::{network::transport::SendingReceivingTransport, pkg::PkgsAux};

  #[tokio::test]
  async fn unit() {
    let mut pa = PkgsAux::from_minimum((), (), ());
    assert_eq!(().send_pkg_recv_decode_contained(&mut (), &mut pa).await.unwrap(), ());
  }
}
