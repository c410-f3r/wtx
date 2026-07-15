use crate::{
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPool, Http2RM},
    push_h2_alpn,
  },
  misc::TcpParams,
  pool::{ResourceManager, SimplePool},
  rng::ChaCha20,
  sync::{Arc, AtomicCell},
  tls::TlsConfig,
};

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct Http2ClientPoolBuilder<EX, TM> {
  disable_auto_sni: bool,
  executor: EX,
  hrp: HttpRecvParams,
  len: usize,
  rng: ChaCha20,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<TM>>,
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM> {
  /// Creates a new builder with the maximum number of connections delimited by `len`.
  ///
  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn new(
    executor: EX,
    len: usize,
    rng: ChaCha20,
    mut tls_config: TlsConfig<TM>,
  ) -> crate::Result<Self> {
    push_h2_alpn(&mut tls_config)?;
    Ok(Self {
      disable_auto_sni: false,
      executor,
      hrp: HttpRecvParams::with_optioned_params(),
      len,
      rng,
      tcp_params: TcpParams::default(),
      tls_config: tls_config.into(),
    })
  }
}

#[cfg(feature = "tokio")]
impl<TM> Http2ClientPoolBuilder<crate::executor::TokioExecutor, TM> {
  /// Calls [`Self::new`] using the elements provided by the tokio project
  #[inline]
  pub fn tokio(len: usize, tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    use crate::rng::CryptoSeedableRng as _;
    Self::new(
      crate::executor::TokioExecutor::default(),
      len,
      ChaCha20::from_std_random()?,
      tls_config,
    )
  }
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM> {
  /// If `true`, then the SNI TLS extension won't be added with the hostname of the URL.
  #[inline]
  pub const fn disable_auto_sni_mut(&mut self) -> &mut bool {
    &mut self.disable_auto_sni
  }

  /// See [`HttpRecvParams`].
  #[inline]
  pub const fn http_conn_params_mut(&mut self) -> &mut HttpRecvParams {
    &mut self.hrp
  }

  /// See [`ChaCha20`].
  #[inline]
  pub const fn rng_mut(&mut self) -> &mut ChaCha20 {
    &mut self.rng
  }

  /// See [`TcpParams`].
  #[inline]
  pub const fn tcp_params_mut(&mut self) -> &mut TcpParams {
    &mut self.tcp_params
  }
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM>
where
  Http2RM<EX, TM>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> Http2ClientPool<EX, TM> {
    Http2ClientPool {
      pool: SimplePool::new(
        self.len,
        Http2RM {
          disable_auto_sni: self.disable_auto_sni,
          executor: self.executor,
          hrp: self.hrp,
          rng: AtomicCell::new(self.rng),
          tcp_params: self.tcp_params,
          tls_config: self.tls_config,
        },
      ),
    }
  }
}
