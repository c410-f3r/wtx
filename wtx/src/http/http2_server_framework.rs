//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#![expect(clippy::infinite_loop, reason = "expected behavior of a server")]

#[macro_use]
mod macros;

mod cors_middleware;
mod dyn_params;
mod endpoint;
mod endpoint_node;
mod http2_server_framework_error;
mod http_router;
mod json_reply;
mod methods;
mod middleware;
mod path;
mod path_params;
mod redirect;
mod res_finalizer;
mod route_match;
mod state;
#[cfg(test)]
mod tests;
mod verbatim_params;

use crate::{
  collections::ArrayVectorCopy,
  executor::{Executor, Runtime as _, TcpListener as _, TcpStream as _},
  http::{
    AutoStream, HttpRecvParams, ManualStream, MsgBufferString, OperationMode, Request, Response,
    push_h2_alpn,
  },
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus, ServerStream},
  misc::{TcpParams, Uri},
  rng::{CryptoRng, CryptoSeedableRng, SeedableRng as _, Xorshift64},
  stream::{Stream, StreamReader, StreamWriter},
  sync::Arc,
  tls::{TlsAcceptor, TlsConfig, TlsMode},
};
use core::{mem, net::IpAddr, num::NonZeroUsize};
pub use cors_middleware::{CorsMiddleware, OriginResponse};
pub use dyn_params::DynParams;
pub use endpoint::Endpoint;
pub use endpoint_node::EndpointNode;
pub use http_router::HttpRouter;
pub use http2_server_framework_error::Http2ServerFrameworkError;
pub use json_reply::*;
pub use methods::{
  delete::{Delete, delete},
  get::{Get, get},
  json::{Json, json},
  patch::{Patch, patch},
  post::{Post, post},
  put::{Put, put},
  web_socket::{WebSocket, web_socket},
};
pub use middleware::Middleware;
pub use path::Path;
pub use path_params::PathParams;
pub use redirect::Redirect;
pub use res_finalizer::ResFinalizer;
pub use route_match::RouteMatch;
pub use state::{State, StateClean, StateGeneric, StateTest};
pub use verbatim_params::VerbatimParams;

type ConnRsltTy<EX, TM, ER> = Option<(
  LocalStream<EX, TM>,
  Result<(ArrayVectorCopy<RouteMatch, 4>, Option<MsgBufferString>), ER>,
)>;
type LocalStream<EX, TM> =
  ServerStream<<<EX as Executor>::TcpStream as Stream>::WriteHalfOwned, TM>;
type WriteHalf<EX> = <<EX as Executor>::TcpStream as Stream>::WriteHalfOwned;

/// HTTP/2 Server Framework
#[derive(Debug)]
pub struct Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  data: DA,
  error_cb: EC,
  executor: EX,
  hrc: HttpRecvParams,
  local_runtime_cb: RC,
  local_runtimes: Option<NonZeroUsize>,
  rng: RNG,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<TM>>,
}

impl<ERR, EX, RNG, TM>
  Http2ServerFramework<
    (),
    fn(ERR),
    EX,
    fn() -> Result<<EX as Executor>::LocalRuntime, ERR>,
    RNG,
    TM,
  >
where
  ERR: From<crate::Error>,
  EX: Executor,
{
  /// Taking aside the provided parameters, everything else is set to default values.
  ///
  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn new(executor: EX, rng: RNG, mut tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    push_h2_alpn(&mut tls_config)?;
    let error_cb: fn(_) = |_| {};
    let local_runtime_cb: fn() -> _ = || Ok(EX::LocalRuntime::new()?);
    Ok(Self {
      data: (),
      error_cb,
      executor,
      hrc: HttpRecvParams::with_optioned_params(),
      local_runtime_cb,
      local_runtimes: None,
      rng,
      tcp_params: TcpParams::default(),
      tls_config: tls_config.into(),
    })
  }
}

#[cfg(feature = "tokio")]
impl<ERR, TM>
  Http2ServerFramework<
    (),
    fn(ERR),
    crate::executor::TokioExecutor,
    fn() -> Result<<crate::executor::TokioExecutor as Executor>::LocalRuntime, ERR>,
    crate::rng::ChaCha20,
    TM,
  >
where
  ERR: From<crate::Error>,
{
  /// Calls [`Self::new`] using the elements provided by the tokio project
  #[inline]
  pub fn tokio(tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    Self::new(
      crate::executor::TokioExecutor::default(),
      crate::rng::ChaCha20::from_std_random()?,
      tls_config,
    )
  }
}

impl<DA, EC, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  /// Mutable Random Number Generator.
  #[inline]
  pub const fn rng_mut(&mut self) -> &mut RNG {
    &mut self.rng
  }

  /// Shared data that is cloned across connections and streams.
  #[inline]
  pub fn set_data<_DA>(self, value: _DA) -> Http2ServerFramework<_DA, EC, EX, RC, RNG, TM> {
    Http2ServerFramework {
      data: value,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      local_runtime_cb: self.local_runtime_cb,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// Sets the error callback function.
  #[inline]
  pub fn set_error_cb<_EC>(self, value: _EC) -> Http2ServerFramework<DA, _EC, EX, RC, RNG, TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: value,
      executor: self.executor,
      hrc: self.hrc,
      local_runtime_cb: self.local_runtime_cb,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// See [`HttpRecvParams`].
  #[inline]
  #[must_use]
  pub fn set_http_recv_params(mut self, value: HttpRecvParams) -> Self {
    self.hrc = value;
    self
  }

  /// Allows the tweaking of the chosen runtime.
  #[inline]
  pub fn set_local_runtime_cb<_RC>(
    self,
    value: _RC,
  ) -> Http2ServerFramework<DA, EC, EX, _RC, RNG, TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      local_runtime_cb: value,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// The number of spawned runtimes. Shouldn't be greater than the number of available threads.
  ///
  /// Only works when calling [`Self::run_in_threads`]. If [`None`], defaults to the number of threads.
  #[inline]
  #[must_use]
  pub fn set_local_runtimes(mut self, value: Option<NonZeroUsize>) -> Self {
    self.local_runtimes = value;
    self
  }

  /// See [`TcpParams`].
  #[inline]
  #[must_use]
  pub fn set_tcp_params(mut self, value: TcpParams) -> Self {
    self.tcp_params = value;
    self
  }
}

impl<DA, EC, ER, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM>
where
  EC: Clone + Fn(ER) + Send + 'static,
  DA: Clone + Send + Sync + 'static,
  EX: Clone + Executor + Send + 'static,
  EX::TcpListener: Send + 'static,
  EX::TcpStream: Send + 'static,
  RC: Clone + Fn() -> Result<EX::LocalRuntime, ER> + 'static,
  RNG: CryptoRng + CryptoSeedableRng + Send + 'static,
  TM: TlsMode + Send + Sync + 'static,
{
  /// Starts the server distributing connections across multiple tasks.
  ///
  /// You must call this method from within an existing async environment. Preferably, a
  /// multi-thread environment.
  #[cfg(feature = "nightly")]
  #[inline]
  pub async fn run<EN, M>(
    mut self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  ) -> Result<(), ER>
  where
    EX::SpawnFuture<()>: Send,
    EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
    <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
    <EX::TcpStream as StreamReader>::read(..): Send,
    <EX::TcpStream as StreamWriter>::write_all(..): Send,
    <EX::TcpStream as StreamWriter>::write_all_vectored(..): Send,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read_skip(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
  {
    let http_router = Arc::new(hr);
    let uri = Uri::new(addr);
    let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), self.tcp_params).await?;
    let xorshift = &mut Xorshift64::from_simple_seed()?;
    loop {
      let Ok(cp) = conn_params(
        (&self.data, &self.error_cb, &self.executor, self.hrc),
        (&mut self.rng, self.tcp_params, &self.tls_config),
        (&http_router, &listener, xorshift),
      )
      .await
      else {
        continue;
      };
      let conn_fut = async move {
        let fut = http2::<EX, _, _>(cp.hrc, cp.rng, cp.stream, cp.tls_config, cp.xorshift);
        let (frame_reader, http2, ip) = match fut.await {
          Err(err) => {
            (cp.error_cb)(err.into());
            return;
          }
          Ok(elem) => elem,
        };
        let _frame_reader_jh = cp.executor.spawn(frame_reader);
        loop {
          let (server_stream, headers_aux, opt) = match conn_rslt::<ER, EX, TM>(
            http2.stream(|req, _| stream_cb::<_, _, _, EX, _, _>(&cp.http_router, req)).await,
          ) {
            Ok(Some(el)) => el,
            Ok(None) => break,
            Err(err) => {
              http2.send_go_away(Http2ErrorCode::NoError).await;
              (cp.error_cb)(err);
              break;
            }
          };
          let stream_data = cp.data.clone();
          let stream_error_cb = cp.error_cb.clone();
          let stream_http_router = cp.http_router.clone();
          let _stream_jh = cp.executor.spawn(stream_fut::<DA, EC, EN, ER, EX, M, TM>(
            headers_aux,
            ip,
            opt,
            server_stream,
            stream_data,
            stream_error_cb,
            stream_http_router,
          ));
        }
      };
      let _conn_jh = self.executor.spawn(conn_fut);
    }
  }

  /// Starts the server using a runtime-per-thread architecture.
  ///
  /// Contrary to the other methods, this method internally creates a runtime according to the
  /// specified `local_runtimes` value.
  ///
  /// You must call this method inside the main thread to allow the interruption of the remaining
  /// threads once an error arises.
  #[cfg(feature = "std")]
  #[inline]
  pub fn run_in_threads<EN, M>(
    mut self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    RC: Send,
    RNG: Send,
    TM: Send + Sync,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
    <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
  {
    use crate::collections::Vector;
    use alloc::string::String;

    let runtimes = if let Some(elem) = self.local_runtimes {
      elem.get()
    } else {
      std::thread::available_parallelism().map_err(crate::Error::from)?.get()
    };
    let http_router = Arc::new(hr);
    let mut join_handles = Vector::<std::thread::JoinHandle<Result<(), ER>>>::new();
    for _ in 0..runtimes {
      let thread_data = self.data.clone();
      let thread_error_cb = self.error_cb.clone();
      let thread_executor = self.executor.clone();
      let thread_hrc = self.hrc;
      let thread_http_router = http_router.clone();
      let thread_local_runtime_cb: RC = self.local_runtime_cb.clone();
      let mut thread_rng = RNG::from_crypto_rng(&mut self.rng)?;
      let thread_tcp_params = self.tcp_params;
      let thread_tls_config = self.tls_config.clone();
      let thread_uri = Uri::new(String::from(addr));
      join_handles.push(std::thread::spawn(move || {
        thread_local_runtime_cb()?.block_on(async move {
          let hostname = thread_uri.hostname_with_implied_port();
          let listener = EX::TcpListener::bind(hostname, thread_tcp_params).await?;
          let xorshift = &mut Xorshift64::from_simple_seed()?;
          loop {
            let Ok(cp) = conn_params(
              (&thread_data, &thread_error_cb, &thread_executor, thread_hrc),
              (&mut thread_rng, thread_tcp_params, &thread_tls_config),
              (&thread_http_router, &listener, xorshift),
            )
            .await
            else {
              continue;
            };
            let _conn_jh = thread_executor.spawn_local(async move {
              let fut = http2::<EX, _, _>(cp.hrc, cp.rng, cp.stream, cp.tls_config, cp.xorshift);
              let (frame_reader, http2, ip) = match fut.await {
                Err(err) => {
                  (cp.error_cb)(err.into());
                  return;
                }
                Ok(elem) => elem,
              };
              let _frame_reader_jh = cp.executor.spawn_local(frame_reader);
              loop {
                let (server_stream, headers_aux, opt) = match conn_rslt::<ER, EX, TM>(
                  http2.stream(|req, _| stream_cb::<_, _, _, EX, _, _>(&cp.http_router, req)).await,
                ) {
                  Ok(Some(el)) => el,
                  Ok(None) => break,
                  Err(err) => {
                    http2.send_go_away(Http2ErrorCode::NoError).await;
                    (cp.error_cb)(err);
                    break;
                  }
                };
                let stream_data = cp.data.clone();
                let stream_error_cb = cp.error_cb.clone();
                let stream_http_router = cp.http_router.clone();
                let _stream_jh = cp.executor.spawn_local(stream_fut::<DA, EC, EN, ER, EX, M, TM>(
                  headers_aux,
                  ip,
                  opt,
                  server_stream,
                  stream_data,
                  stream_error_cb,
                  stream_http_router,
                ));
              }
            });
          }
        })
      }))?;
    }
    for join_handle in join_handles {
      join_handle.join().map_err(crate::Error::from)??;
    }
    Ok(())
  }

  /// Starts the server handling all connections on the current thread.
  ///
  /// You must call this method from within an existing async environment.
  #[inline]
  pub async fn run_local<EN, M>(
    mut self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
    <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
  {
    let http_router = Arc::new(hr);
    let uri = Uri::new(addr);
    let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), self.tcp_params).await?;
    let xorshift = &mut Xorshift64::from_simple_seed()?;
    loop {
      let Ok(cp) = conn_params(
        (&self.data, &self.error_cb, &self.executor, self.hrc),
        (&mut self.rng, self.tcp_params, &self.tls_config),
        (&http_router, &listener, xorshift),
      )
      .await
      else {
        continue;
      };
      let _conn_jh = self.executor.spawn_local(async move {
        let fut = http2::<EX, _, _>(cp.hrc, cp.rng, cp.stream, cp.tls_config, cp.xorshift);
        let (frame_reader, http2, ip) = match fut.await {
          Err(err) => {
            (cp.error_cb)(err.into());
            return;
          }
          Ok(elem) => elem,
        };
        let _frame_reader_jh = cp.executor.spawn_local(frame_reader);
        loop {
          let (server_stream, headers_aux, opt) = match conn_rslt::<ER, EX, TM>(
            http2.stream(|req, _| stream_cb::<_, _, _, EX, _, _>(&cp.http_router, req)).await,
          ) {
            Ok(Some(el)) => el,
            Ok(None) => break,
            Err(err) => {
              http2.send_go_away(Http2ErrorCode::NoError).await;
              (cp.error_cb)(err);
              break;
            }
          };
          let stream_data = cp.data.clone();
          let stream_error_cb = cp.error_cb.clone();
          let stream_http_router = cp.http_router.clone();
          let _stream_jh = cp.executor.spawn_local(stream_fut::<DA, EC, EN, ER, EX, M, TM>(
            headers_aux,
            ip,
            opt,
            server_stream,
            stream_data,
            stream_error_cb,
            stream_http_router,
          ));
        }
      });
    }
  }
}

struct ConnParams<DA, EC, EN, ER, EX, M, RNG, TM>
where
  EX: Executor,
{
  data: DA,
  error_cb: EC,
  executor: EX,
  hrc: HttpRecvParams,
  http_router: Arc<HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>>,
  rng: RNG,
  stream: EX::TcpStream,
  tls_config: Arc<TlsConfig<TM>>,
  xorshift: Xorshift64,
}

#[inline]
async fn conn_params<DA, EC, EN, ER, EX, M, RNG, TM>(
  (data, error_cb, executor, hrc): (&DA, &EC, &EX, HttpRecvParams),
  (rng, tcp_params, tls_config): (&mut RNG, TcpParams, &Arc<TlsConfig<TM>>),
  (http_router, listener, xorshift): (
    &Arc<HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>>,
    &EX::TcpListener,
    &mut Xorshift64,
  ),
) -> crate::Result<ConnParams<DA, EC, EN, ER, EX, M, RNG, TM>>
where
  EC: Clone,
  DA: Clone,
  EX: Clone + Executor,
  RNG: CryptoRng + CryptoSeedableRng,
  TM: TlsMode + Send + 'static,
{
  Ok(ConnParams {
    data: data.clone(),
    error_cb: error_cb.clone(),
    executor: executor.clone(),
    hrc,
    http_router: http_router.clone(),
    rng: RNG::from_crypto_rng(rng)?,
    stream: listener.accept(tcp_params).await?.0,
    tls_config: tls_config.clone(),
    xorshift: Xorshift64::from_rng(xorshift)?,
  })
}

#[inline]
fn conn_rslt<ER, EX, TM>(
  rslt: crate::Result<ConnRsltTy<EX, TM, ER>>,
) -> Result<
  Option<(LocalStream<EX, TM>, ArrayVectorCopy<RouteMatch, 4>, Option<MsgBufferString>)>,
  ER,
>
where
  ER: From<crate::Error>,
  EX: Executor,
  TM: TlsMode,
{
  if let Some((server_stream, local_rslt)) = rslt.map_err(ER::from)? {
    let (headers_aux, opt) = local_rslt?;
    Ok(Some((server_stream, headers_aux, opt)))
  } else {
    Ok(None)
  }
}

#[inline]
async fn http2<EX, RNG, TM>(
  hrc: HttpRecvParams,
  mut rng: RNG,
  stream: EX::TcpStream,
  tls_config: Arc<TlsConfig<TM>>,
  mut xorshift: Xorshift64,
) -> crate::Result<(impl Future<Output = ()>, Http2<WriteHalf<EX>, TM, false>, IpAddr)>
where
  EX: Executor,
  RNG: CryptoRng,
  TM: TlsMode,
{
  let ip = stream.peer_addr()?.ip();
  let tar = TlsAcceptor::new(&*tls_config, &mut rng, stream).accept().await?;
  let split = tar.tls_stream.into_split()?;
  let tuple = Http2::accept(Http2Buffer::new(&mut xorshift), hrc, split).await?;
  Ok((tuple.0, tuple.1, ip))
}

#[inline]
fn log_req(_peer: &IpAddr, _req: &Request<MsgBufferString>) {
  let _method = _req.method.strings().custom[0];
  let _path = _req.msg_data.uri.path();
  _debug!(r#"{_peer} "{_method} {_path}""#,);
}

#[expect(clippy::needless_pass_by_value, reason = "doesn't matter")]
#[inline]
fn stream_cb<DA, EN, ER, EX, M, TM>(
  http_router: &HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  req: Request<&mut MsgBufferString>,
) -> Result<(ArrayVectorCopy<RouteMatch, 4>, Option<MsgBufferString>), ER>
where
  ER: From<crate::Error>,
  EX: Executor,
{
  let rslt = *http_router.router.find(req.msg_data.uri.path())?.data();
  Ok(match rslt.1 {
    OperationMode::Auto => (rslt.0, None),
    OperationMode::Manual => (rslt.0, Some(mem::take(req.msg_data))),
  })
}

#[inline]
async fn stream_fut<DA, EC, EN, ER, EX, M, TM>(
  headers_aux: ArrayVectorCopy<RouteMatch, 4>,
  ip: IpAddr,
  opt: Option<MsgBufferString>,
  mut server_stream: LocalStream<EX, TM>,
  stream_data: DA,
  stream_error_cb: EC,
  stream_http_router: Arc<HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>>,
) where
  EC: Fn(ER),
  EN: EndpointNode<DA, ER, LocalStream<EX, TM>>,
  ER: From<crate::Error>,
  EX: Executor,
  M: Middleware<DA, ER>,
  TM: TlsMode,
{
  let stream_fun = async {
    if let Some(local_rrb) = opt {
      let req = Request::http2(server_stream.method(), local_rrb);
      log_req(&ip, &req);
      let manual_stream = ManualStream {
        data: stream_data,
        peer: ip,
        protocol: server_stream.protocol(),
        req,
        stream: server_stream.clone(),
      };
      stream_http_router.en.manual(manual_stream, (0, &headers_aux)).await?;
      return Ok(());
    }
    let (hrs, local_rrb) = server_stream.recv_req().await?;
    if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
      return Ok(());
    }
    let req = local_rrb.into_http2_request(server_stream.method());
    log_req(&ip, &req);
    let mut auto_stream = AutoStream::new(stream_data, ip, server_stream.protocol(), req);
    let status = stream_http_router.auto(&mut auto_stream, (0, &headers_aux)).await?;
    let enc_buffer = mem::take(&mut auto_stream.req.msg_data.uri).into_inner();
    let _ = server_stream
      .send_res(
        &mut enc_buffer.into_bytes().into(),
        Response::new(auto_stream.req.msg_data, status),
      )
      .await?;
    Ok::<_, ER>(())
  };
  let stream_fun_rslt = stream_fun.await;
  let _rslt = server_stream.common().clear().await;
  if let Err(err) = stream_fun_rslt {
    stream_error_cb(err);
  }
}
