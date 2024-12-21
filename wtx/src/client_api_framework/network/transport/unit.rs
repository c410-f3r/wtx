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

impl<DRSR> RecievingTransport<DRSR> for () {
  #[inline]
  async fn recv<A>(
    &mut self,
    _: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..0)
  }
}

impl<DRSR> SendingTransport<DRSR> for () {
  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, ()>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, ()>,
  {
    manage_before_sending_related(pkg, pkgs_aux, self).await?;
    manage_after_sending_related(pkg, pkgs_aux).await?;
    Ok(())
  }
}

/// Does absolutely nothing. Good for demonstration purposes.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{network::transport::SendingRecievingTransport, pkg::PkgsAux};
/// let _ =
///   ().send_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ())).await?;
/// # Ok(()) }
/// ```
impl<DRSR> Transport<DRSR> for () {
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Params = ();
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::client_api_framework::{network::transport::SendingRecievingTransport, pkg::PkgsAux};

  #[tokio::test]
  async fn unit() {
    let mut pa = PkgsAux::from_minimum((), (), ());
    let mut trans = ();
    assert_eq!(trans.send_recv_decode_contained(&mut (), &mut pa).await.unwrap(), ());
  }
}
