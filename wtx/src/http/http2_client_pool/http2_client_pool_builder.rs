use crate::{
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPool, Http2RM},
  },
  pool::{ResourceManager, SimplePool},
  rng::ChaCha20,
  sync::AtomicCell,
  tls::{TlsConfig, TlsModeStrict},
};

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct Http2ClientPoolBuilder<EX, TM> {
  executor: EX,
  hrp: HttpRecvParams,
  len: usize,
  tls_config: TlsConfig<'static>,
  tls_mode: TM,
}

impl<EX> Http2ClientPoolBuilder<EX, TlsModeStrict> {
  /// Creates a new builder with the maximum number of connections delimited by `len`.
  #[inline]
  pub const fn new(executor: EX, len: usize, tls_config: TlsConfig<'static>) -> Self {
    Self {
      executor,
      hrp: HttpRecvParams::with_optioned_params(),
      len,
      tls_config,
      tls_mode: TlsModeStrict,
    }
  }
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM> {
  /// See [`HttpRecvParams`].
  #[inline]
  pub const fn http_conn_params(mut self, value: HttpRecvParams) -> Self {
    self.hrp = value;
    self
  }

  /// See [`TlsConfig`].
  #[inline]
  pub const fn tls_config_mut(&mut self) -> &mut TlsConfig<'static> {
    &mut self.tls_config
  }

  /// TLS mode
  #[inline]
  pub fn tls_mode<_TM>(self, value: _TM) -> Http2ClientPoolBuilder<EX, _TM> {
    Http2ClientPoolBuilder {
      executor: self.executor,
      hrp: self.hrp,
      len: self.len,
      tls_config: self.tls_config,
      tls_mode: value,
    }
  }
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM>
where
  Http2RM<EX, TM>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self, rng: ChaCha20) -> Http2ClientPool<EX, TM> {
    Http2ClientPool {
      pool: SimplePool::new(
        self.len,
        Http2RM {
          executor: self.executor,
          hrp: self.hrp,
          rng: AtomicCell::new(rng),
          tls_mode: self.tls_mode,
        },
      ),
    }
  }
}
