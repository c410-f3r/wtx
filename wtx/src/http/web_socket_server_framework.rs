use crate::{
  collection::Vector,
  executor::{Executor, Runtime, TcpListener},
  http::{HttpError, Router},
  misc::{TcpParams, Uri},
  rng::{ChaCha20, CryptoRng, CryptoSeedableRng},
  stream::{StreamReader, StreamWriter},
  sync::Arc,
  tls::{TlsAcceptor, TlsConfig, TlsMode, TlsModeVerified},
  web_socket::{Compression, WebSocket, WebSocketAcceptor, WebSocketBuffer},
};
use alloc::string::String;
use core::num::NonZeroUsize;

type LocalWs<CO, EX, TM> = WebSocket<
  <CO as Compression<false>>::NegotiatedCompression,
  <EX as Executor>::TcpStream,
  TM,
  WebSocketBuffer,
  false,
>;

pub struct WebSocketServerFramework<CO, EC, EX, RC, RNG, TM> {
  compression: CO,
  err_cb: EC,
  executor: EX,
  rng: RNG,
  runtime_cb: RC,
  tcp_params: TcpParams,
  threads: Option<NonZeroUsize>,
  tls_config: Option<Arc<TlsConfig<'static>>>,
  tls_mode: TM,
}

impl<EX>
  WebSocketServerFramework<
    (),
    fn(crate::Error),
    EX,
    fn() -> crate::Result<<EX as Executor>::LocalRuntime>,
    ChaCha20,
    TlsModeVerified,
  >
where
  EX: Executor,
{
  #[inline]
  pub fn new(executor: EX) -> crate::Result<Self> {
    let err_cb: fn(_) = |_| {};
    let runtime_cb: fn() -> _ = || EX::LocalRuntime::optioned();
    Ok(Self {
      compression: (),
      err_cb,
      executor,
      rng: ChaCha20::from_std_random()?,
      runtime_cb,
      tcp_params: TcpParams::default(),
      threads: None,
      tls_config: None,
      tls_mode: TlsModeVerified,
    })
  }
}

impl<CO, EC, EX, RC, RNG, TM> WebSocketServerFramework<CO, EC, EX, RC, RNG, TM> {
  /// Sets the compression algorithm.
  #[inline]
  pub fn set_compression<_C>(self, value: _C) -> WebSocketServerFramework<_C, EC, EX, RC, RNG, TM> {
    WebSocketServerFramework {
      compression: value,
      err_cb: self.err_cb,
      executor: self.executor,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
    }
  }

  /// Sets the error callback function.
  #[inline]
  pub fn set_error_cb<_EC>(self, value: _EC) -> WebSocketServerFramework<CO, _EC, EX, RC, RNG, TM> {
    WebSocketServerFramework {
      compression: self.compression,
      err_cb: value,
      executor: self.executor,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
    }
  }

  #[inline]
  pub fn set_runtime_cb<_RC>(
    self,
    value: _RC,
  ) -> WebSocketServerFramework<CO, EC, EX, _RC, RNG, TM> {
    WebSocketServerFramework {
      compression: self.compression,
      err_cb: self.err_cb,
      executor: self.executor,
      rng: self.rng,
      runtime_cb: value,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
    }
  }

  /// See [`TcpParams`].
  #[inline]
  pub fn set_tcp_params(mut self, value: TcpParams) -> Self {
    self.tcp_params = value;
    self
  }

  #[inline]
  pub fn set_threads(mut self, value: Option<NonZeroUsize>) -> Self {
    self.threads = value;
    self
  }

  #[inline]
  pub fn set_tls_config(mut self, value: TlsConfig<'static>) -> Self {
    self.tls_config = Some(Arc::new(value));
    self
  }

  /// Sets the TLS mode.
  #[inline]
  pub fn set_tls_mode<_TM>(self, value: _TM) -> WebSocketServerFramework<CO, EC, EX, RC, RNG, _TM> {
    WebSocketServerFramework {
      compression: self.compression,
      err_cb: self.err_cb,
      executor: self.executor,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: value,
    }
  }
}

impl<CO, EC, ER, EX, RC, RNG, TM> WebSocketServerFramework<CO, EC, EX, RC, RNG, TM>
where
  CO: Clone + Compression<false> + Send + 'static,
  CO::NegotiatedCompression: Send,
  EC: Clone + Fn(ER) + Send + 'static,
  ER: From<crate::Error> + Send + 'static,
  EX: Clone + Executor + Send + 'static,
  EX::TcpListener: Send + 'static,
  EX::TcpStream: Send + 'static,
  RC: Clone + Fn() -> Result<EX::LocalRuntime, ER> + Send + 'static,
  RNG: CryptoRng + CryptoSeedableRng + Send + 'static,
  TM: Default + TlsMode + Send + 'static,
  <EX::TcpListener as TcpListener>::accept(..): Send,
  <EX::TcpStream as StreamReader>::read(..): Send,
  <EX::TcpStream as StreamWriter>::write_all(..): Send,
{
  /// Starts the server listening on the specified address.
  pub async fn run<WSR>(self, addr: &str, wsr: WSR) -> Result<(), ER>
  where
    WSR: WebSocketRouter<CO, ER, EX, TM> + Send + Sync + 'static,
    WSR::call(..): Send,
  {
    let web_socket_router = Arc::new(wsr);
    let Self { compression, err_cb, executor, rng, tcp_params, tls_config, .. } = self;
    do_run(
      addr,
      compression,
      err_cb,
      executor,
      rng,
      build_matcher(&*web_socket_router)?,
      tcp_params,
      tls_config_actual::<TM>(tls_config)?,
      web_socket_router,
    )
    .await
  }

  #[cfg(feature = "std")]
  pub fn run_in_threads<WSR>(self, addr: &str, wsr: WSR) -> Result<(), ER>
  where
    WSR: WebSocketRouter<CO, ER, EX, TM> + Send + Sync + 'static,
    WSR::call(..): Send,
  {
    let Self {
      compression,
      err_cb,
      executor,
      mut rng,
      runtime_cb,
      tcp_params,
      threads,
      tls_config,
      tls_mode: _,
    } = self;
    let tls_config_actual = tls_config_actual::<TM>(tls_config)?;
    let number = if let Some(elem) = threads {
      elem.get()
    } else {
      cfg_select! {
        feature = "std" => std::thread::available_parallelism().map_err(crate::Error::from)?.get(),
        _ => 1usize
      }
    };
    let web_socket_router = Arc::new(wsr);
    let router = build_matcher(&*web_socket_router)?;
    let mut join_handles = Vector::new();
    for _ in 0..number {
      let thread_addr = String::from(addr);
      let thread_compression = compression.clone();
      let thread_err_cb = err_cb.clone();
      let thread_executor = executor.clone();
      let thread_router = router.clone();
      let thread_rng = RNG::from_crypto_rng(&mut rng)?;
      let thread_runtime_cb = runtime_cb.clone();
      let thread_tcp_params = tcp_params.clone();
      let thread_tls_config = tls_config_actual.clone();
      let thread_web_socket_router = web_socket_router.clone();
      join_handles.push(std::thread::spawn(move || {
        thread_runtime_cb()?.block_on(do_run(
          thread_addr.as_str(),
          thread_compression,
          thread_err_cb,
          thread_executor,
          thread_rng,
          thread_router,
          thread_tcp_params,
          thread_tls_config,
          thread_web_socket_router,
        ))
      }))?;
    }
    for join_handle in join_handles {
      join_handle.join().map_err(crate::Error::from)??;
    }
    Ok(())
  }
}

fn build_matcher<CO, ER, EX, TM, WSR>(
  web_socket_router: &WSR,
) -> Result<Arc<Router<u8>>, crate::Error>
where
  CO: Compression<false>,
  EX: Executor,
  WSR: WebSocketRouter<CO, ER, EX, TM>,
{
  let mut matcher = Router::new();
  {
    let mut builder = matcher.builder();
    let mut idx: u8 = 0;
    for path in web_socket_router.paths() {
      builder.add(&path.try_into()?, idx)?;
      idx = idx.wrapping_add(1);
    }
  }
  Ok(Arc::new(matcher))
}

async fn do_run<CO, EC, ER, EX, RNG, TM, WSR>(
  addr: &str,
  compression: CO,
  err_cb: EC,
  executor: EX,
  mut rng: RNG,
  router: Arc<Router<u8>>,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<'static>>,
  web_socket_router: Arc<WSR>,
) -> Result<(), ER>
where
  CO: Clone + Compression<false> + Send + 'static,
  CO::NegotiatedCompression: Send,
  EC: Clone + Fn(ER) + Send + 'static,
  ER: From<crate::Error> + Send + 'static,
  EX: Executor,
  EX::TcpStream: Send + 'static,
  RNG: CryptoRng + CryptoSeedableRng + Send + 'static,
  TM: Default + TlsMode + Send + 'static,
  WSR: WebSocketRouter<CO, ER, EX, TM> + Send + Sync + 'static,
  WSR::call(..): Send,
  <EX::TcpListener as TcpListener>::accept(..): Send,
  <EX::TcpStream as StreamReader>::read(..): Send,
  <EX::TcpStream as StreamWriter>::write_all(..): Send,
{
  let uri = Uri::new(addr);
  let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), tcp_params).await?;
  loop {
    let conn_compression = compression.clone();
    let conn_err_cb = err_cb.clone();
    let conn_router = router.clone();
    let mut conn_rng = RNG::from_crypto_rng(&mut rng)?;
    let conn_stream = listener.accept().await?.0;
    let conn_tls_config = tls_config.clone();
    let conn_web_socket_router = web_socket_router.clone();
    let _jh = executor.spawn(async move {
      let fun = async move {
        let mut path = String::new();
        let web_socket = WebSocketAcceptor::default()
          .compression(conn_compression)
          .req(|req| {
            if let Some(elem) = req.path {
              path.push_str(elem);
            }
            crate::Result::Ok(true)
          })
          .accept(&mut conn_rng, TlsAcceptor::new(conn_stream, TM::default()), &conn_tls_config)
          .await?;
        conn_web_socket_router.call(&conn_router, path, web_socket).await?;
        Ok::<_, ER>(())
      };
      if let Err(err) = fun.await {
        conn_err_cb(err);
      }
    });
  }
}

#[inline]
fn tls_config_actual<TM>(
  tls_config: Option<Arc<TlsConfig<'static>>>,
) -> crate::Result<Arc<TlsConfig<'static>>>
where
  TM: TlsMode,
{
  Ok(if TM::TY.require_certs() {
    tls_config.ok_or(HttpError::TlsConnectionRequireTlsConfig)?
  } else {
    Arc::new(TlsConfig::uncertified())
  })
}

pub trait WebSocketRouter<CO, ER, EX, TM>
where
  CO: Compression<false>,
  EX: Executor,
{
  fn call(
    &self,
    matcher: &Router<u8>,
    path: String,
    ws: LocalWs<CO, EX, TM>,
  ) -> impl Future<Output = Result<(), ER>>;

  fn paths(&self) -> impl ExactSizeIterator<Item = &'static str>;
}
