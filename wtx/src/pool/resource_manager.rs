/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Auxiliary data used by the [`Self::create`] method.
  type CreateAux: ?Sized;
  /// Any custom error.
  type Error: From<crate::Error>;
  /// Auxiliary data used by the [`Self::recycle`] method.
  type RecycleAux: ?Sized;
  /// Any pool resource.
  type Resource;

  /// Creates a new resource instance based on the contents of this manager.
  fn create(
    &self,
    aux: &Self::CreateAux,
  ) -> impl Future<Output = Result<Self::Resource, Self::Error>>;

  /// If a resource is in an invalid state.
  fn is_invalid(&self, resource: &Self::Resource) -> bool;

  /// Re-creates a new valid instance. Should be called if `resource` is invalid.
  fn recycle(
    &self,
    aux: &Self::RecycleAux,
    resource: &mut Self::Resource,
  ) -> impl Future<Output = Result<(), Self::Error>>;
}

impl ResourceManager for () {
  type CreateAux = ();
  type Error = crate::Error;
  type RecycleAux = ();
  type Resource = ();

  #[inline]
  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Ok(())
  }

  #[inline]
  fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  #[inline]
  async fn recycle(&self, _: &Self::RecycleAux, _: &mut Self::Resource) -> Result<(), Self::Error> {
    Ok(())
  }
}

/// Manages generic resources that are always valid and don't require logic for recycling.
#[derive(Debug)]
pub struct SimpleRM<F> {
  /// Create callback
  pub cb: F,
}

impl<F> SimpleRM<F> {
  /// Shortcut constructor
  #[inline]
  pub const fn new(cb: F) -> Self {
    Self { cb }
  }
}

impl<E, F, R> ResourceManager for SimpleRM<F>
where
  E: From<crate::Error>,
  F: Fn() -> Result<R, E>,
{
  type CreateAux = ();
  type Error = E;
  type RecycleAux = ();
  type Resource = R;

  #[inline]
  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    (self.cb)()
  }

  #[inline]
  fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  #[inline]
  async fn recycle(&self, _: &Self::RecycleAux, _: &mut Self::Resource) -> Result<(), Self::Error> {
    Ok(())
  }
}

#[cfg(feature = "postgres-pool")]
pub(crate) mod database {
  macro_rules! _executor {
    ($uri_secret:expr, |$config:ident, $uri:ident| $cb:expr) => {{
      $uri_secret
        .peek(&mut Vector::new(), async |secret| {
          // SAFETY: URI is a string.
          let string = unsafe { core::str::from_utf8_unchecked(&*secret) };
          let $uri = crate::misc::UriRef::new(string);
          let config_rslt = crate::database::client::postgres::Config::from_uri(&$uri);
          let $config = config_rslt?;
          $cb.await
        })?
        .await?
    }};
  }

  use crate::{
    collections::Vector,
    database::{
      DEFAULT_MAX_STMTS, DbClient as _,
      client::postgres::{ClientBuffer, PostgresClient},
    },
    executor::TcpStream,
    misc::{Secret, SecretContext, TcpParams},
    pool::ResourceManager,
    rng::ChaCha20,
    sync::{Arc, AtomicCell},
    tls::{TlsConfig, TlsConnector, TlsMode},
  };
  use core::{marker::PhantomData, mem};

  /// Manages generic database executors.
  #[derive(Debug)]
  pub struct PostgresRM<E, S, TM> {
    max_stmts: usize,
    phantom: PhantomData<(fn() -> E, S)>,
    rng: AtomicCell<ChaCha20>,
    secret: Secret,
    tcp_params: TcpParams,
    tls_config: Arc<TlsConfig<TM>>,
  }

  impl<E, S, TM> PostgresRM<E, S, TM> {
    /// Generic resource manager
    #[inline]
    pub fn new(
      mut rng: ChaCha20,
      secret_context: SecretContext,
      tls_config: Arc<TlsConfig<TM>>,
      uri: &mut [u8],
    ) -> crate::Result<Self> {
      let secret = Secret::new(uri, &mut rng, secret_context)?;
      Ok(Self {
        max_stmts: DEFAULT_MAX_STMTS,
        phantom: PhantomData,
        rng: AtomicCell::new(rng),
        secret,
        tcp_params: TcpParams::default(),
        tls_config,
      })
    }
  }

  impl<E, S, TM> ResourceManager for PostgresRM<E, S, TM>
  where
    E: From<crate::Error>,
    S: TcpStream,
    TM: TlsMode,
  {
    type CreateAux = ();
    type Error = E;
    type RecycleAux = ();
    type Resource = PostgresClient<E, S, TM>;

    #[inline]
    async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let client_buffer = ClientBuffer::new(self.max_stmts, &mut &self.rng);
      let rng = &mut &self.rng;
      let tls_config = &*self.tls_config;
      Ok(_executor!(&self.secret, |postgres_config, uri| {
        let stream = S::connect(uri.hostname_with_implied_port(), self.tcp_params).await?;
        let tls_connector = TlsConnector::new(tls_config, rng, stream);
        PostgresClient::connect(client_buffer, &postgres_config, tls_connector)
      }))
    }

    #[inline]
    fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.connection_state().is_closed()
    }

    #[inline]
    async fn recycle(
      &self,
      _: &Self::RecycleAux,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let mut client_buffer = ClientBuffer::new(self.max_stmts, &mut &self.rng);
      let rng = &mut &self.rng;
      let tls_config = &*self.tls_config;
      mem::swap(&mut client_buffer, &mut resource.cb);
      *resource = _executor!(&self.secret, |postgres_config, uri| {
        let stream = S::connect(uri.hostname_with_implied_port(), self.tcp_params).await?;
        let tls_connector = TlsConnector::new(tls_config, rng, stream);
        PostgresClient::connect(client_buffer, &postgres_config, tls_connector)
      });
      Ok(())
    }
  }
}
