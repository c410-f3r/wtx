#![expect(clippy::infinite_loop, reason = "expected behavior of a server")]

use crate::{
  collections::Vector,
  executor::{Executor, Runtime as _},
  http::Router,
  misc::{TcpParams, Uri},
  rng::{CryptoRng, CryptoSeedableRng},
  stream::{StreamReader, StreamWriter, TcpListener},
  sync::Arc,
  tls::{TlsAcceptor, TlsConfig, TlsMode},
  web_socket::{WebSocket, WebSocketAcceptor, WsCompression},
};
use alloc::string::String;
use core::num::NonZeroUsize;

type LocalWebSocket<CO, EX, TM> = WebSocket<
  <CO as WsCompression<false>>::NegotiatedCompression,
  <EX as Executor>::TcpStream,
  TM,
  false,
>;

/// WebSocket Server Framework
#[derive(Debug)]
pub struct WebSocketServerFramework<CO, EC, EX, RC, RNG, TM> {
  compression: CO,
  error_cb: EC,
  executor: EX,
  local_runtime_cb: RC,
  local_runtimes: Option<NonZeroUsize>,
  rng: RNG,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<TM>>,
}

impl<ER, EX, RNG, TM>
  WebSocketServerFramework<
    (),
    fn(ER),
    EX,
    fn() -> Result<<EX as Executor>::LocalRuntime, ER>,
    RNG,
    TM,
  >
where
  ER: From<crate::Error>,
  EX: Executor,
{
  /// Taking aside the provided parameters, everything else is set to default values.
  #[inline]
  pub fn new(executor: EX, rng: RNG, tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    let error_cb: fn(_) = |_| {};
    let local_runtime_cb: fn() -> _ = || Ok(EX::LocalRuntime::new()?);
    Ok(Self {
      compression: (),
      error_cb,
      executor,
      local_runtime_cb,
      local_runtimes: None,
      rng,
      tcp_params: TcpParams::default(),
      tls_config: tls_config.into(),
    })
  }
}

#[cfg(feature = "tokio")]
impl<ER, TM>
  WebSocketServerFramework<
    (),
    fn(ER),
    crate::executor::TokioExecutor,
    fn() -> Result<<crate::executor::TokioExecutor as Executor>::LocalRuntime, ER>,
    crate::rng::ChaCha20,
    TM,
  >
where
  ER: From<crate::Error>,
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

impl<CO, EC, EX, RC, RNG, TM> WebSocketServerFramework<CO, EC, EX, RC, RNG, TM> {
  /// Sets the compression algorithm.
  #[inline]
  pub fn set_compression<_C>(self, value: _C) -> WebSocketServerFramework<_C, EC, EX, RC, RNG, TM> {
    WebSocketServerFramework {
      compression: value,
      error_cb: self.error_cb,
      executor: self.executor,
      local_runtime_cb: self.local_runtime_cb,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// Sets the error callback function.
  #[inline]
  pub fn set_error_cb<_EC>(self, value: _EC) -> WebSocketServerFramework<CO, _EC, EX, RC, RNG, TM> {
    WebSocketServerFramework {
      compression: self.compression,
      error_cb: value,
      executor: self.executor,
      local_runtime_cb: self.local_runtime_cb,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// Allows the tweaking of the chosen runtime.
  ///
  /// Only works when calling [`Self::run_in_threads`].
  #[inline]
  pub fn set_local_runtime_cb<_RC>(
    self,
    value: _RC,
  ) -> WebSocketServerFramework<CO, EC, EX, _RC, RNG, TM> {
    WebSocketServerFramework {
      compression: self.compression,
      error_cb: self.error_cb,
      executor: self.executor,
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

  /// See [`TlsConfig`].
  #[inline]
  pub fn set_tls_config<_TM>(
    self,
    value: TlsConfig<_TM>,
  ) -> WebSocketServerFramework<CO, EC, EX, RC, RNG, _TM> {
    WebSocketServerFramework {
      compression: self.compression,
      error_cb: self.error_cb,
      executor: self.executor,
      local_runtime_cb: self.local_runtime_cb,
      local_runtimes: self.local_runtimes,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: Arc::new(value),
    }
  }
}

impl<CO, EC, ER, EX, RC, RNG, TM> WebSocketServerFramework<CO, EC, EX, RC, RNG, TM>
where
  CO: Clone + WsCompression<false> + 'static,
  EC: Clone + Fn(ER) + 'static,
  ER: From<crate::Error> + 'static,
  EX: Clone + Executor + 'static,
  EX::TcpListener: 'static,
  EX::TcpStream: 'static,
  RC: Clone + Fn() -> Result<EX::LocalRuntime, ER> + 'static,
  RNG: CryptoRng + CryptoSeedableRng + 'static,
  TM: TlsMode + 'static,
{
  /// Starts the server distributing connections across multiple tasks.
  ///
  /// You must call this method from within an existing async environment. Preferably, a
  /// multi-thread environment.
  #[cfg(feature = "nightly")]
  #[inline]
  pub async fn run<WSR>(mut self, addr: &str, wsr: WSR) -> Result<(), ER>
  where
    CO: Send,
    CO::NegotiatedCompression: Send + Sync,
    EC: Send,
    ER: Send,
    EX: Send,
    EX::TcpListener: Send,
    EX::TcpStream: Send,
    RC: Send,
    RNG: Send,
    TM: Send + Sync,
    WSR: WebSocketRouter<CO, ER, EX, TM> + Send + Sync + 'static,
    WSR::call(..): Send,
    <EX::TcpListener as TcpListener>::accept(..): Send,
    <EX::TcpStream as StreamReader>::read(..): Send,
    <EX::TcpStream as StreamWriter>::write_all(..): Send,
    <EX::TcpStream as StreamWriter>::write_all_vectored(..): Send,
  {
    let uri = Uri::new(addr);
    let web_socket_router = Arc::new(wsr);
    let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), self.tcp_params).await?;
    let router = build_matcher(&*web_socket_router)?;
    loop {
      let Ok(cp) = conn_params::<CO, EC, EX, RNG, TM, WSR>(
        (&self.compression, &self.error_cb),
        (&mut self.rng, self.tcp_params, &self.tls_config),
        &listener,
        &router,
        &web_socket_router,
      )
      .await
      else {
        continue;
      };
      let _jh = self.executor.spawn(conn_fut(cp));
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
  pub fn run_in_threads<WSR>(mut self, addr: &str, wsr: WSR) -> Result<(), ER>
  where
    CO: Send,
    EC: Send,
    ER: Send,
    EX: Send,
    RC: Send,
    RNG: Send,
    TM: Send + Sync,
    WSR: WebSocketRouter<CO, ER, EX, TM> + Send + Sync + 'static,
  {
    let runtimes = if let Some(elem) = self.local_runtimes {
      elem.get()
    } else {
      std::thread::available_parallelism().map_err(crate::Error::from)?.get()
    };
    let web_socket_router = Arc::new(wsr);
    let router = build_matcher(&*web_socket_router)?;
    let mut join_handles = Vector::<std::thread::JoinHandle<Result<(), ER>>>::new();
    for _ in 0..runtimes {
      let thread_comp = self.compression.clone();
      let thread_error_cb = self.error_cb.clone();
      let thread_executor = self.executor.clone();
      let thread_local_runtime_cb = self.local_runtime_cb.clone();
      let mut thread_rng = RNG::from_crypto_rng(&mut self.rng)?;
      let thread_router = router.clone();
      let thread_tcp_params = self.tcp_params;
      let thread_tls_config = self.tls_config.clone();
      let thread_uri = Uri::new(String::from(addr));
      let thread_web_socket_router = web_socket_router.clone();
      join_handles.push(std::thread::spawn(move || {
        thread_local_runtime_cb()?.block_on(async move {
          let hostname = thread_uri.hostname_with_implied_port();
          let listener = EX::TcpListener::bind(hostname, thread_tcp_params).await?;
          loop {
            let Ok(cp) = conn_params::<CO, EC, EX, RNG, TM, WSR>(
              (&thread_comp, &thread_error_cb),
              (&mut thread_rng, thread_tcp_params, &thread_tls_config),
              &listener,
              &thread_router,
              &thread_web_socket_router,
            )
            .await
            else {
              continue;
            };
            let _jh = thread_executor.spawn_local(conn_fut(cp));
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
  pub async fn run_local<WSR>(mut self, addr: &str, wsr: WSR) -> Result<(), ER>
  where
    WSR: WebSocketRouter<CO, ER, EX, TM> + 'static,
  {
    let web_socket_router = Arc::new(wsr);
    let router = build_matcher(&*web_socket_router)?;
    let uri = Uri::new(addr);
    let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), self.tcp_params).await?;
    loop {
      let Ok(cp) = conn_params::<CO, EC, EX, RNG, TM, WSR>(
        (&self.compression, &self.error_cb),
        (&mut self.rng, self.tcp_params, &self.tls_config),
        &listener,
        &router,
        &web_socket_router,
      )
      .await
      else {
        continue;
      };
      let _jh = self.executor.spawn_local(conn_fut(cp));
    }
  }
}

/// Routes path according to the set of user-provided functions and paths.
pub trait WebSocketRouter<CO, ER, EX, TM>
where
  CO: WsCompression<false>,
  EX: Executor,
{
  /// Calls user-provided functions.
  fn call(
    &self,
    matcher: &Router<u8>,
    path: String,
    ws: LocalWebSocket<CO, EX, TM>,
  ) -> impl Future<Output = Result<(), ER>>;

  /// All user registered paths
  fn paths(&self) -> impl ExactSizeIterator<Item = &'static str>;
}

struct ConnParams<CO, EC, EX, RNG, TM, WSR>
where
  EX: Executor,
{
  compression: CO,
  error_cb: EC,
  rng: RNG,
  router: Arc<Router<u8>>,
  stream: EX::TcpStream,
  tls_config: Arc<TlsConfig<TM>>,
  web_socket_router: Arc<WSR>,
}

fn build_matcher<CO, ER, EX, TM, WSR>(web_socket_router: &WSR) -> crate::Result<Arc<Router<u8>>>
where
  CO: WsCompression<false>,
  EX: Executor,
  WSR: WebSocketRouter<CO, ER, EX, TM>,
{
  let mut matcher = Router::new();
  {
    let mut builder = matcher.builder();
    let mut idx: u8 = 0;
    for path in web_socket_router.paths() {
      let _ = builder.add(&path.try_into()?, idx)?;
      idx = idx.wrapping_add(1);
    }
  }
  Ok(Arc::new(matcher))
}

#[inline]
async fn conn_fut<CO, EC, ER, EX, RNG, TM, WSR>(conn_params: ConnParams<CO, EC, EX, RNG, TM, WSR>)
where
  CO: WsCompression<false>,
  EC: Fn(ER),
  ER: From<crate::Error>,
  EX: Executor,
  RNG: CryptoRng + CryptoSeedableRng,
  TM: TlsMode,
  WSR: WebSocketRouter<CO, ER, EX, TM>,
{
  let fun = async {
    let mut path = String::new();
    let web_socket = WebSocketAcceptor::default()
      .set_compression(conn_params.compression)
      .set_req(|req| {
        if let Some(elem) = req.path {
          path.push_str(elem);
        }
        crate::Result::Ok(true)
      })
      .accept(TlsAcceptor::new(&*conn_params.tls_config, conn_params.rng, conn_params.stream))
      .await?;
    conn_params.web_socket_router.call(&conn_params.router, path, web_socket).await?;
    Ok::<_, ER>(())
  };
  if let Err(err) = fun.await {
    (conn_params.error_cb)(err);
  }
}

async fn conn_params<CO, EC, EX, RNG, TM, WSR>(
  (compression, error_cb): (&CO, &EC),
  (rng, tcp_params, tls_config): (&mut RNG, TcpParams, &Arc<TlsConfig<TM>>),
  listener: &EX::TcpListener,
  router: &Arc<Router<u8>>,
  web_socket_router: &Arc<WSR>,
) -> crate::Result<ConnParams<CO, EC, EX, RNG, TM, WSR>>
where
  CO: Clone,
  EC: Clone,
  EX: Executor,
  RNG: CryptoRng + CryptoSeedableRng,
{
  Ok(ConnParams {
    compression: compression.clone(),
    error_cb: error_cb.clone(),
    rng: RNG::from_crypto_rng(rng)?,
    router: router.clone(),
    stream: listener.accept(tcp_params).await?.0,
    tls_config: tls_config.clone(),
    web_socket_router: web_socket_router.clone(),
  })
}
