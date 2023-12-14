use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{transport::Transport, TransportGroup},
    pkg::{Package, PkgsAux},
  },
  misc::AsyncBounds,
};
use core::ops::Range;

/// Does absolutely nothing. Good for demonstration purposes.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{network::transport::Transport, pkg::PkgsAux};
/// let _ =
///   ().send_retrieve_and_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ())).await?;
/// # Ok(()) }
/// ```
impl<DRSR> Transport<DRSR> for ()
where
  DRSR: AsyncBounds,
{
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Params = ();

  #[inline]
  async fn send<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, ()>,
  ) -> Result<(), P::Error>
  where
    P: AsyncBounds + Package<DRSR, ()>,
  {
    manage_before_sending_related(pkg, pkgs_aux, self).await?;
    manage_after_sending_related(pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_and_retrieve<P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<P::Api, DRSR, ()>,
  ) -> Result<Range<usize>, P::Error>
  where
    P: AsyncBounds + Package<DRSR, ()>,
  {
    self.send(pkg, pkgs_aux).await?;
    Ok(0..0)
  }
}

#[cfg(test)]
mod tests {
  use crate::client_api_framework::{network::transport::Transport, pkg::PkgsAux};

  #[tokio::test]
  async fn unit() {
    let mut pa = PkgsAux::from_minimum((), (), ());
    let mut trans = ();
    assert_eq!(trans.send_retrieve_and_decode_contained(&mut (), &mut pa).await.unwrap(), ());
  }
}
