use crate::{
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPool, Http2RM},
    push_h2_alpn,
  },
  net::TcpParams,
  pool::{ResourceManager, SimplePool},
  rng::ChaCha20,
  sync::{AsyncMutex, AtomicCell},
  tls::TlsConfig,
};

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct Http2ClientPoolBuilder<AUX, EX, TCX> {
  aux_fn: fn() -> AUX,
  disable_auto_sni: bool,
  executor: EX,
  hrp: HttpRecvParams,
  len: usize,
  rng: ChaCha20,
  tcp_params: TcpParams,
  tls_config: TlsConfig<TCX>,
}

impl<EX, TCX> Http2ClientPoolBuilder<(), EX, TCX> {
  /// Creates a new builder with the maximum number of connections delimited by `len`.
  ///
  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn new(
    executor: EX,
    len: usize,
    rng: ChaCha20,
    mut tls_config: TlsConfig<TCX>,
  ) -> crate::Result<Self> {
    push_h2_alpn(&mut tls_config)?;
    Ok(Self {
      aux_fn: || {},
      disable_auto_sni: false,
      executor,
      hrp: HttpRecvParams::with_optioned_params(),
      len,
      rng,
      tcp_params: TcpParams::default(),
      tls_config,
    })
  }
}

#[cfg(feature = "tokio")]
impl<TCX> Http2ClientPoolBuilder<(), crate::executor::TokioExecutor, TCX> {
  /// Calls [`Self::new`] using the elements provided by the tokio project
  #[inline]
  pub fn tokio(len: usize, tls_config: TlsConfig<TCX>) -> crate::Result<Self> {
    use crate::rng::CryptoSeedableRng as _;
    Self::new(
      crate::executor::TokioExecutor::default(),
      len,
      ChaCha20::from_std_random()?,
      tls_config,
    )
  }
}

impl<AUX, EX, TCX> Http2ClientPoolBuilder<AUX, EX, TCX> {
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

  /// Function that returns auxiliary data.
  #[inline]
  pub fn set_aux_fn<_AUX>(self, value: fn() -> _AUX) -> Http2ClientPoolBuilder<_AUX, EX, TCX> {
    Http2ClientPoolBuilder {
      aux_fn: value,
      disable_auto_sni: self.disable_auto_sni,
      executor: self.executor,
      hrp: self.hrp,
      len: self.len,
      rng: self.rng,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// See [`TcpParams`].
  #[inline]
  pub const fn tcp_params_mut(&mut self) -> &mut TcpParams {
    &mut self.tcp_params
  }
}

impl<AUX, EX, TCX> Http2ClientPoolBuilder<AUX, EX, TCX>
where
  Http2RM<AUX, EX, TCX>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> Http2ClientPool<AUX, EX, TCX> {
    Http2ClientPool {
      pool: SimplePool::new(
        self.len,
        Http2RM {
          aux_fn: self.aux_fn,
          disable_auto_sni: self.disable_auto_sni,
          executor: self.executor,
          hrp: self.hrp,
          rng: AtomicCell::new(self.rng),
          tcp_params: self.tcp_params,
          tls_config: AsyncMutex::new(self.tls_config),
        },
      ),
    }
  }
}
