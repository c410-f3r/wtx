use crate::client_api_framework::{
  Api,
  misc::{
    log_req, manage_after_sending_bytes, manage_after_sending_pkg, manage_before_sending_bytes,
    manage_before_sending_pkg,
  },
  network::{
    TransportGroup,
    transport::{ReceivingTransport, SendingTransport, Transport, local_send_bytes},
  },
  pkg::{Package, PkgsAux},
};

impl<TP> ReceivingTransport<TP> for () {
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    _: &mut PkgsAux<A, DRSR, TP>,
    _: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    Ok(())
  }
}

impl<TP> SendingTransport<TP> for () {
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: &[u8],
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    manage_before_sending_bytes(pkgs_aux).await?;
    let local_bytes = local_send_bytes(bytes, &pkgs_aux.bytes_buffer, pkgs_aux.send_bytes_buffer);
    log_req::<_, TP>(local_bytes, pkgs_aux.log_body.1, self);
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
    log_req::<_, TP>(&pkgs_aux.bytes_buffer, pkgs_aux.log_body.1, self);
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
  type ReqId = ();
}
