pub trait AcceptorBackend {
  type Acceptor;
  type WithoutClientAuth;

  fn without_client_auth(&mut self) -> Self::WithoutClientAuth;

  fn build_with_cert_chain_and_priv_key(
    self,
    wca: Self::WithoutClientAuth,
    cert_chain: &[u8],
    is_http2: bool,
    priv_key: &[u8],
  ) -> crate::Result<Self::Acceptor>;
}

impl AcceptorBackend for () {
  type Acceptor = ();
  type WithoutClientAuth = ();

  #[inline]
  fn without_client_auth(&mut self) -> Self {
    ()
  }

  #[inline]
  fn build_with_cert_chain_and_priv_key(
    self,
    _: Self::WithoutClientAuth,
    _: &[u8],
    _: bool,
    _: &[u8],
  ) -> crate::Result<Self::Acceptor> {
    Ok(())
  }
}
