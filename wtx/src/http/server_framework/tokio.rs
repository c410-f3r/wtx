use crate::{
  http::{
    server_framework::{
      ConnAux, PathManagement, ReqAux, ReqMiddleware, ResMiddleware, Router, ServerFramework,
    },
    Headers, OptionedServer, ReqResBuffer,
  },
  http2::{Http2Buffer, ServerStreamTokio},
  misc::Rng,
};
use std::sync::Arc;
use tokio::net::{tcp::OwnedWriteHalf, TcpStream};

impl<CA, CAC, E, P, RA, RAC, REQM, RESM> ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM>
where
  CA: Clone + ConnAux + Send + 'static,
  CAC: Clone + Fn() -> CA::Init + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  P: PathManagement<CA, E, RA, manage_path(..): Send> + Send + 'static,
  RA: ReqAux + Send + 'static,
  RAC: Clone + Fn() -> RA::Init + Send + 'static,
  REQM: ReqMiddleware<CA, E, RA, apply_req_middleware(..): Send> + Send + 'static,
  RESM: ResMiddleware<CA, E, RA, apply_res_middleware(..): Send> + Send + 'static,
  Arc<Router<CA, E, P, RA, REQM, RESM>>: Send,
  Router<CA, E, P, RA, REQM, RESM>: Send,
  for<'any> &'any Arc<Router<CA, E, P, RA, REQM, RESM>>: Send,
  for<'any> &'any Router<CA, E, P, RA, REQM, RESM>: Send,
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
    let Self { _ca_cb: ca_cb, _cp: cp, _ra_cb: ra_cb, _router: router } = self;
    OptionedServer::tokio_high_http2(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp._to_hp())),
      err_cb,
      Self::manual_tokio,
      move || Ok(((ra_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
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
    let Self { _ca_cb: ca_cb, _cp: cp, _ra_cb: ra_cb, _router: router } = self;
    OptionedServer::tokio_high_http2(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp._to_hp())),
      err_cb,
      Self::manual_tokio_rustls,
      move || Ok(((ra_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
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
    _: CA,
    _: (impl Fn() -> RA::Init, Arc<Router<CA, E, P, RA, REQM, RESM>>),
    _: Headers,
    _: ServerStreamTokio<Http2Buffer, OwnedWriteHalf, false>,
  ) -> Result<(), E> {
    Err(E::from(crate::Error::ClosedConnection))
  }

  #[cfg(feature = "tokio-rustls")]
  #[inline]
  async fn manual_tokio_rustls(
    _: CA,
    _: (impl Fn() -> RA::Init, Arc<Router<CA, E, P, RA, REQM, RESM>>),
    _: Headers,
    _: ServerStreamTokio<
      Http2Buffer,
      tokio::io::WriteHalf<tokio_rustls::server::TlsStream<TcpStream>>,
      false,
    >,
  ) -> Result<(), E> {
    Err(E::from(crate::Error::ClosedConnection))
  }
}
