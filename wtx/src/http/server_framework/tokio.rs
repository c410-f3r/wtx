use crate::{
  http::{
    ManualServerStreamTokio, OptionedServer, ReqResBuffer, Request,
    server_framework::{
      ConnAux, EndpointNode, Middleware, RouteMatch, Router, ServerFramework, StreamAux,
    },
  },
  http2::{Http2Buffer, Http2DataTokio, ServerStream},
  misc::{Arc, ArrayVector, SeedableRng},
};
use tokio::net::tcp::OwnedWriteHalf;

type Stream = ServerStream<Http2DataTokio<Http2Buffer, OwnedWriteHalf, false>>;
#[cfg(feature = "tokio-rustls")]
type StreamRustls = ServerStream<
  Http2DataTokio<
    Http2Buffer,
    tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
    false,
  >,
>;

impl<CA, CACB, CBP, E, EN, M, SA, SACB> ServerFramework<CA, CACB, CBP, E, EN, M, Stream, SA, SACB>
where
  CA: Clone + ConnAux + Send + 'static,
  CACB: Clone + Fn(CBP) -> CA::Init + Send + 'static,
  CBP: Clone + SeedableRng + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, Stream, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send,
  SA: StreamAux + Send + 'static,
  SACB: Clone + Fn(&mut CA) -> SA::Init + Send + 'static,
  Arc<Router<CA, E, EN, M, Stream, SA>>: Send,
  for<'any> &'any CA: Send,
  for<'any> &'any Router<CA, E, EN, M, Stream, SA>: Send,
  for<'any> &'any SA: Send,
{
  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn tokio(
    self,
    host: &str,
    conn_error_cb: impl Clone + Fn(E) + Send + 'static,
    headers_cb: impl Clone + Fn(Request<&mut ReqResBuffer>) -> Result<(), E> + Send + Sync + 'static,
    stream_error_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()> {
    let Self { _ca_cb, _cbp, _cp, _sa_cb, _router } = self;
    OptionedServer::http2_tokio(
      ((), host, _cbp, _router),
      |local_rng| {
        *local_rng = CBP::from_rng(local_rng);
      },
      |_, stream| async move { Ok(stream.into_split()) },
      conn_error_cb,
      move |mut local_rng| {
        let hb = Http2Buffer::new(&mut local_rng);
        Ok((CA::conn_aux(_ca_cb(local_rng))?, hb, _cp._to_hp()))
      },
      move |ca| Ok((SA::stream_aux(_sa_cb(ca))?, ReqResBuffer::empty())),
      move |_, local_router, _, req, _| {
        let rslt = Self::_route_params(req.rrd.uri.path(), local_router)?;
        headers_cb(req)?;
        Ok(((rslt.0, Arc::clone(local_router)), rslt.1))
      },
      stream_error_cb,
      Self::_auto,
      Self::tokio_manual,
    )
    .await
  }

  #[inline]
  async fn tokio_manual(
    headers_aux: (ArrayVector<RouteMatch, 4>, Arc<Router<CA, E, EN, M, Stream, SA>>),
    manual_stream: ManualServerStreamTokio<CA, Http2Buffer, SA, OwnedWriteHalf>,
  ) -> Result<(), E> {
    headers_aux.1.en.manual(manual_stream, (0, &headers_aux.0)).await?;
    Ok(())
  }
}

#[cfg(feature = "tokio-rustls")]
impl<CA, CACB, CBP, E, EN, M, SA, SACB>
  ServerFramework<CA, CACB, CBP, E, EN, M, StreamRustls, SA, SACB>
where
  CA: Clone + ConnAux + Send + 'static,
  CACB: Clone + Fn(CBP) -> CA::Init + Send + 'static,
  CBP: Clone + SeedableRng + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, StreamRustls, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send,
  SA: StreamAux + Send + 'static,
  SACB: Clone + Fn(&mut CA) -> SA::Init + Send + 'static,
  Arc<Router<CA, E, EN, M, StreamRustls, SA>>: Send,
  for<'any> &'any CA: Send,
  for<'any> &'any Router<CA, E, EN, M, StreamRustls, SA>: Send,
  for<'any> &'any SA: Send,
{
  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[inline]
  pub async fn tokio_rustls(
    self,
    (cert_chain, priv_key): (&[u8], &[u8]),
    host: &str,
    conn_error_cb: impl Clone + Fn(E) + Send + 'static,
    headers_cb: impl Clone + Fn(Request<&mut ReqResBuffer>) -> Result<(), E> + Send + Sync + 'static,
    stream_error_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()> {
    let Self { _ca_cb, _cbp, _cp, _sa_cb, _router } = self;
    let tls_acceptor = crate::misc::TokioRustlsAcceptor::without_client_auth()
      .http2()
      .build_with_cert_chain_and_priv_key(cert_chain, priv_key)?;
    OptionedServer::http2_tokio(
      (tls_acceptor, host, _cbp, _router),
      |local_rng| {
        *local_rng = CBP::from_rng(local_rng);
      },
      |acceptor, stream| async move { Ok(tokio::io::split(acceptor.accept(stream).await?)) },
      conn_error_cb,
      move |mut local_rng| {
        let hb = Http2Buffer::new(&mut local_rng);
        Ok((CA::conn_aux(_ca_cb(local_rng))?, hb, _cp._to_hp()))
      },
      move |ca| Ok((SA::stream_aux(_sa_cb(ca))?, ReqResBuffer::empty())),
      move |_, local_router, _, req, _| {
        let rslt = Self::_route_params(req.rrd.uri.path(), local_router)?;
        headers_cb(req)?;
        Ok(((rslt.0, Arc::clone(local_router)), rslt.1))
      },
      stream_error_cb,
      Self::_auto,
      Self::tokio_rustls_manual,
    )
    .await
  }

  #[inline]
  async fn tokio_rustls_manual(
    headers_aux: (ArrayVector<RouteMatch, 4>, Arc<Router<CA, E, EN, M, StreamRustls, SA>>),
    manual_stream: ManualServerStreamTokio<
      CA,
      Http2Buffer,
      SA,
      tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
    >,
  ) -> Result<(), E> {
    headers_aux.1.en.manual(manual_stream, (0, &headers_aux.0)).await?;
    Ok(())
  }
}
