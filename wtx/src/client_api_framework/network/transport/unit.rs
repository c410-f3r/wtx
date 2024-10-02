use crate::client_api_framework::{
  misc::{manage_after_sending_related, manage_before_sending_related},
  network::{transport::Transport, TransportGroup},
  pkg::{Package, PkgsAux},
  Api,
};
use core::ops::Range;

/// Does absolutely nothing. Good for demonstration purposes.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{network::transport::Transport, pkg::PkgsAux};
/// let _ =
///   ().send_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ())).await?;
/// # Ok(()) }
/// ```
impl<DRSR> Transport<DRSR> for () {
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Params = ();

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

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, ()>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, ()>,
  {
    self.send(pkg, pkgs_aux).await?;
    Ok(0..0)
  }
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::client_api_framework::{network::transport::Transport, pkg::PkgsAux};

  #[tokio::test]
  async fn unit() {
    let mut pa = PkgsAux::from_minimum((), (), ());
    let mut trans = ();
    assert_eq!(trans.send_recv_decode_contained(&mut (), &mut pa).await.unwrap(), ());
  }
}
