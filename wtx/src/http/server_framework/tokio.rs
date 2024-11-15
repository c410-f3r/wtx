use crate::{
  http::{
    server_framework::{
      ConnAux, EndpointNode, Middleware, RouteMatch, Router, ServerFramework, StreamAux,
    },
    ManualServerStreamTokio, ManualStream, OptionedServer, ReqResBuffer, Request,
  },
  http2::{Http2Buffer, Http2DataTokio, ServerStream},
  misc::{Arc, ArrayVector, Rng},
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

impl<CA, CAC, E, EN, M, SA, SAC> ServerFramework<CA, CAC, E, EN, M, Stream, SA, SAC>
where
  CA: Clone + ConnAux + Send + 'static,
  CAC: Clone + Fn() -> CA::Init + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, Stream, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send + 'static,
  SA: StreamAux + Send + 'static,
  SAC: Clone + Fn() -> SA::Init + Send + 'static,
  Arc<Router<CA, E, EN, M, Stream, SA>>: Send,
  Router<CA, E, EN, M, Stream, SA>: Send,
  for<'any> &'any (SAC, Arc<Router<CA, E, EN, M, Stream, SA>>): Send,
  for<'any> &'any CA: Send,
  for<'any> &'any M: Send,
  for<'any> &'any Router<CA, E, EN, M, Stream, SA>: Send,
{
  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn tokio<RNG>(
    self,
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    operation_mode: impl Clone + Fn(Request<&mut ReqResBuffer>) -> Result<(), E> + Send + Sync + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { _ca_cb, _cp, _sa_cb, _router } = self;
    OptionedServer::http2_tokio(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(_ca_cb())?, Http2Buffer::new(rng.clone()), _cp._to_hp())),
      err_cb,
      Self::tokio_manual,
      move |_, _, req, sa| {
        let rslt = Self::_route_params(req.rrd.uri.path(), &sa.1)?;
        operation_mode(req)?;
        Ok(rslt)
      },
      move || Ok(((_sa_cb.clone(), Arc::clone(&_router)), ReqResBuffer::empty())),
      (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
    )
    .await
  }

  #[inline]
  async fn tokio_manual(
    headers_aux: ArrayVector<RouteMatch, 4>,
    manual_stream: ManualServerStreamTokio<
      CA,
      Http2Buffer,
      (impl Fn() -> SA::Init, Arc<Router<CA, E, EN, M, Stream, SA>>),
      OwnedWriteHalf,
    >,
  ) -> Result<(), E> {
    let router_manual_stream = ManualStream {
      conn_aux: manual_stream.conn_aux,
      peer: manual_stream.peer,
      protocol: manual_stream.protocol,
      req: manual_stream.req,
      stream: manual_stream.stream,
      stream_aux: SA::stream_aux(manual_stream.stream_aux.0())?,
    };
    manual_stream.stream_aux.1.en.manual(router_manual_stream, (0, &headers_aux)).await?;
    Ok(())
  }
}

#[cfg(feature = "tokio-rustls")]
impl<CA, CAC, E, EN, M, SA, SAC> ServerFramework<CA, CAC, E, EN, M, StreamRustls, SA, SAC>
where
  CA: Clone + ConnAux + Send + 'static,
  CAC: Clone + Fn() -> CA::Init + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  EN: EndpointNode<CA, E, StreamRustls, SA, auto(..): Send, manual(..): Send> + Send + 'static,
  M: Middleware<CA, E, SA, req(..): Send, res(..): Send> + Send + 'static,
  M::Aux: Send + 'static,
  SA: StreamAux + Send + 'static,
  SAC: Clone + Fn() -> SA::Init + Send + 'static,
  Arc<Router<CA, E, EN, M, StreamRustls, SA>>: Send,
  Router<CA, E, EN, M, StreamRustls, SA>: Send,
  for<'any> &'any (SAC, Arc<Router<CA, E, EN, M, StreamRustls, SA>>): Send,
  for<'any> &'any CA: Send,
  for<'any> &'any M: Send,
  for<'any> &'any Router<CA, E, EN, M, StreamRustls, SA>: Send,
{
  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[inline]
  pub async fn tokio_rustls<RNG>(
    self,
    (cert_chain, priv_key): (&'static [u8], &'static [u8]),
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
    operation_mode: impl Clone + Fn(Request<&mut ReqResBuffer>) -> Result<(), E> + Send + Sync + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { _ca_cb, _cp, _sa_cb, _router } = self;
    OptionedServer::http2_tokio(
      host,
      Self::_auto,
      move || Ok((CA::conn_aux(_ca_cb())?, Http2Buffer::new(rng.clone()), _cp._to_hp())),
      err_cb,
      Self::tokio_rustls_manual,
      move |_, _, req, sa| {
        let rslt = Self::_route_params(req.rrd.uri.path(), &sa.1)?;
        operation_mode(req)?;
        Ok(rslt)
      },
      move || Ok(((_sa_cb.clone(), Arc::clone(&_router)), ReqResBuffer::empty())),
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
  async fn tokio_rustls_manual(
    headers_aux: ArrayVector<RouteMatch, 4>,
    manual_stream: ManualServerStreamTokio<
      CA,
      Http2Buffer,
      (impl Fn() -> SA::Init, Arc<Router<CA, E, EN, M, StreamRustls, SA>>),
      tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
    >,
  ) -> Result<(), E> {
    let router_manual_stream = ManualStream {
      conn_aux: manual_stream.conn_aux,
      peer: manual_stream.peer,
      protocol: manual_stream.protocol,
      req: manual_stream.req,
      stream: manual_stream.stream,
      stream_aux: SA::stream_aux(manual_stream.stream_aux.0())?,
    };
    manual_stream.stream_aux.1.en.manual(router_manual_stream, (0, &headers_aux)).await?;
    Ok(())
  }
}
