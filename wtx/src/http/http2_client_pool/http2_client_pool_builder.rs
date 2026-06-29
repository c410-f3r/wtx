use crate::{
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPool, Http2RM},
    push_h2_alpn,
  },
  misc::TcpParams,
  pool::{ResourceManager, SimplePool},
  rng::{ChaCha20, CryptoSeedableRng as _},
  sync::{Arc, AtomicCell},
  tls::{Psk, TlsConfig},
};

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct Http2ClientPoolBuilder<EX, TM> {
  executor: EX,
  hrp: HttpRecvParams,
  len: usize,
  psk: Option<AtomicCell<Psk>>,
  rng: ChaCha20,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<TM>>,
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM> {
  /// Creates a new builder with the maximum number of connections delimited by `len`.
  ///
  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn new(executor: EX, len: usize, mut tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    push_h2_alpn(tls_config.alpn_mut())?;
    Ok(Self {
      executor,
      hrp: HttpRecvParams::with_optioned_params(),
      len,
      psk: None,
      rng: ChaCha20::from_std_random()?,
      tcp_params: TcpParams::default(),
      tls_config: tls_config.into(),
    })
  }
}

impl<EX, TM> Http2ClientPoolBuilder<EX, TM> {
  /// See [`HttpRecvParams`].
  #[inline]
  pub const fn http_conn_params_mut(&mut self) -> &mut HttpRecvParams {
    &mut self.hrp
  }

  /// See [`Psk`].
  #[inline]
  pub fn psk_mut(&mut self) -> &mut Option<AtomicCell<Psk>> {
    &mut self.psk
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
          executor: self.executor,
          hrp: self.hrp,
          psk: self.psk,
          rng: AtomicCell::new(self.rng),
          tcp_params: self.tcp_params,
          tls_config: self.tls_config,
        },
      ),
    }
  }
}
