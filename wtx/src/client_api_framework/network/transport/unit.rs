use crate::client_api_framework::{
  misc::{manage_after_sending_related, manage_before_sending_related},
  network::{
    transport::{RecievingTransport, SendingTransport, Transport},
    TransportGroup,
  },
  pkg::{Package, PkgsAux},
  Api,
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
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    manage_before_sending_related(pkg, pkgs_aux, self).await?;
    manage_after_sending_related(pkg, pkgs_aux, self).await?;
    Ok(())
  }
}

/// Does absolutely nothing. Good for demonstration purposes.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{network::transport::SendingReceivingTransport, pkg::PkgsAux};
/// let _ =
///   ().send_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ())).await?;
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
    assert_eq!(().send_recv_decode_contained(&mut (), &mut pa).await.unwrap(), ());
  }
}
