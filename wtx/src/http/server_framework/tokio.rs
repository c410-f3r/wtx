use crate::{
  http::{
    server_framework::{ConnAux, Middleware, PathManagement, Router, ServerFramework, StreamAux},
    ManualServerStreamTokio, OptionedServer, ReqResBuffer, StreamMode,
  },
  http2::Http2Buffer,
  misc::Rng,
};
use alloc::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;

impl<CA, CAC, E, M, P, SA, SAC> ServerFramework<CA, CAC, E, M, P, SA, SAC>
where
  CA: Clone + ConnAux + Send + 'static,
  CAC: Clone + Fn() -> CA::Init + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send + 'static,
  P: PathManagement<CA, E, SA, manage_path(..): Send> + Send + 'static,
  SA: StreamAux + Send + 'static,
  SAC: Clone + Fn() -> SA::Init + Send + 'static,
  Arc<Router<CA, E, M, P, SA>>: Send,
  Router<CA, E, M, P, SA>: Send,
  for<'any> &'any Arc<Router<CA, E, M, P, SA>>: Send,
  for<'any> &'any Router<CA, E, M, P, SA>: Send,
{
  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn listen_tokio<RNG>(
    self,
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { _ca_cb: ca_cb, _cp: cp, _sa_cb: sa_cb, _router: router } = self;
    OptionedServer::http2_tokio(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp._to_hp())),
      err_cb,
      Self::manual_tokio,
      move || Ok(((sa_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
      |_, _, _| Ok(StreamMode::Auto),
      (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
    )
    .await
  }

  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[cfg(feature = "tokio-rustls")]
  #[inline]
  pub async fn listen_tokio_rustls<RNG>(
    self,
    (cert_chain, priv_key): (&'static [u8], &'static [u8]),
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { _ca_cb: ca_cb, _cp: cp, _sa_cb: ra_cb, _router: router } = self;
    OptionedServer::http2_tokio(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp._to_hp())),
      err_cb,
      Self::manual_tokio_rustls,
      move || Ok(((ra_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
      |_, _, _| Ok(StreamMode::Auto),
      (
        || {
          crate::misc::TokioRustlsAcceptor::without_client_auth()
            .http2()
            .build_with_cert_chain_and_priv_key(cert_chain, priv_key)
        },
        |acceptor| acceptor.clone(),
        |acceptor, stream| async move { Ok(tokio::io::split(acceptor.accept(stream).await?)) },
      ),
    )
    .await
  }

  #[inline]
  async fn manual_tokio(
    _: ManualServerStreamTokio<
      CA,
      Http2Buffer,
      (impl Fn() -> SA::Init, Arc<Router<CA, E, M, P, SA>>),
      (),
      OwnedWriteHalf,
    >,
  ) -> Result<(), E> {
    Err(E::from(crate::Error::ClosedConnection))
  }

  #[cfg(feature = "tokio-rustls")]
  #[inline]
  async fn manual_tokio_rustls(
    _: ManualServerStreamTokio<
      CA,
      Http2Buffer,
      (impl Fn() -> SA::Init, Arc<Router<CA, E, M, P, SA>>),
      (),
      tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
    >,
  ) -> Result<(), E> {
    Err(E::from(crate::Error::ClosedConnection))
  }
}
