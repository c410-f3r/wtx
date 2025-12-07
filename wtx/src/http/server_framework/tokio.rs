use crate::{
  collection::ArrayVectorU8,
  http::{
    AutoStream, ManualServerStream, OperationMode, OptionedServer, ReqResBuffer, Request, Response,
    server_framework::{
      ConnAux, EndpointNode, Middleware, RouteMatch, Router, ServerFramework, StreamAux,
      endpoint::Endpoint,
    },
  },
  http2::{Http2Buffer, ServerStream},
  rng::{Rng, SeedableRng},
  sync::Arc,
};
use tokio::net::tcp::OwnedWriteHalf;

type Stream = ServerStream<Http2Buffer, OwnedWriteHalf>;
#[cfg(feature = "tokio-rustls")]
type StreamRustls = ServerStream<
  Http2Buffer,
  tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
>;

impl<CA, CACB, CBP, E, EN, M, S, SA, SACB> ServerFramework<CA, CACB, CBP, E, EN, M, S, SA, SACB>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
  M: Middleware<CA, E, SA>,
  SA: StreamAux,
{
  async fn auto(
    headers_aux: (ArrayVectorU8<RouteMatch, 4>, Arc<Router<CA, E, EN, M, S, SA>>),
    mut auto_stream: AutoStream<CA, SA>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let status_code = headers_aux.1.auto(&mut auto_stream, (0, &headers_aux.0)).await?;
    Ok(Response { rrd: auto_stream.req.rrd, status_code, version: auto_stream.req.version })
  }

  pub(crate) fn route_params(
    path: &str,
    router: &Arc<Router<CA, E, EN, M, S, SA>>,
  ) -> Result<(ArrayVectorU8<RouteMatch, 4>, OperationMode), E> {
    #[cfg(feature = "matchit")]
    return Ok(router._matcher.at(path).map_err(From::from)?.value.clone());
    #[cfg(not(feature = "matchit"))]
    return Ok((
      ArrayVectorU8::new(),
      *router
        ._matcher
        .get(path)
        .ok_or_else(|| crate::http::server_framework::ServerFrameworkError::UnknownPath.into())?,
    ));
  }
}

impl<CA, CACB, CBP, E, EN, M, SA, SACB> ServerFramework<CA, CACB, CBP, E, EN, M, Stream, SA, SACB>
where
  CA: Clone + ConnAux + Send + 'static,
  CACB: Clone + Fn(CBP) -> Result<CA::Init, E> + Send + 'static,
  CBP: Clone + Rng + SeedableRng + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, Stream, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send,
  SA: StreamAux + Send + 'static,
  SACB: Clone + Fn(&mut CA) -> Result<SA::Init, E> + Send + 'static,
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
  ) -> Result<(), E> {
    let Self { _ca_cb, _cbp, _cp, _sa_cb, _router } = self;
    OptionedServer::http2_tokio(
      ((), host, _cbp, _router),
      |local_rng| {
        *local_rng = CBP::from_rng(local_rng)?;
        Ok(())
      },
      |_, stream| async move { Ok(stream.into_split()) },
      conn_error_cb,
      move |mut local_rng| {
        let hb = Http2Buffer::new(&mut local_rng);
        Ok((CA::conn_aux(_ca_cb(local_rng)?)?, hb, _cp._to_hp()))
      },
      move |ca| Ok((SA::stream_aux(_sa_cb(ca)?)?, ReqResBuffer::empty())),
      move |_, local_router, _, req, _| {
        let rslt = Self::route_params(req.rrd.uri.path(), local_router)?;
        headers_cb(req)?;
        Ok(((rslt.0, Arc::clone(local_router)), rslt.1))
      },
      stream_error_cb,
      Self::auto,
      Self::tokio_manual,
    )
    .await
  }

  async fn tokio_manual(
    headers_aux: (ArrayVectorU8<RouteMatch, 4>, Arc<Router<CA, E, EN, M, Stream, SA>>),
    manual_stream: ManualServerStream<CA, Http2Buffer, SA, OwnedWriteHalf>,
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
  CACB: Clone + Fn(CBP) -> Result<CA::Init, E> + Send + 'static,
  CBP: Clone + Rng + SeedableRng + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, StreamRustls, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send,
  SA: StreamAux + Send + 'static,
  SACB: Clone + Fn(&mut CA) -> Result<SA::Init, E> + Send + 'static,
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
  ) -> Result<(), E> {
    let Self { _ca_cb, _cbp, _cp, _sa_cb, _router } = self;
    let tls_acceptor = crate::misc::TokioRustlsAcceptor::without_client_auth()
      .http2()
      .build_with_cert_chain_and_priv_key(cert_chain, priv_key)?;
    OptionedServer::http2_tokio(
      (tls_acceptor, host, _cbp, _router),
      |local_rng| {
        *local_rng = CBP::from_rng(local_rng)?;
        Ok(())
      },
      |acceptor, stream| async move {
        Ok(tokio::io::split(acceptor.accept(stream).await.map_err(crate::Error::from)?))
      },
      conn_error_cb,
      move |mut local_rng| {
        let hb = Http2Buffer::new(&mut local_rng);
        Ok((CA::conn_aux(_ca_cb(local_rng)?)?, hb, _cp._to_hp()))
      },
      move |ca| Ok((SA::stream_aux(_sa_cb(ca)?)?, ReqResBuffer::empty())),
      move |_, local_router, _, req, _| {
        let rslt = Self::route_params(req.rrd.uri.path(), local_router)?;
        headers_cb(req)?;
        Ok(((rslt.0, Arc::clone(local_router)), rslt.1))
      },
      stream_error_cb,
      Self::auto,
      Self::tokio_rustls_manual,
    )
    .await
  }

  async fn tokio_rustls_manual(
    headers_aux: (ArrayVectorU8<RouteMatch, 4>, Arc<Router<CA, E, EN, M, StreamRustls, SA>>),
    manual_stream: ManualServerStream<
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
