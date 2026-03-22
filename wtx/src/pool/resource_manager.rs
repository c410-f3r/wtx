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

#[cfg(all(feature = "postgres", feature = "secret"))]
pub(crate) mod database {
  use crate::{
    collection::Vector,
    database::{
      DEFAULT_MAX_STMTS, Executor,
      client::postgres::{ExecutorBuffer, PostgresExecutor},
    },
    misc::{Secret, SecretContext},
    pool::ResourceManager,
    rng::{ChaCha20, CryptoSeedableRng},
    sync::AtomicCell,
  };
  use core::{marker::PhantomData, mem};

  /// Manages generic database executors.
  #[derive(Debug)]
  pub struct PostgresRM<E, S> {
    _certs: Option<Vector<u8>>,
    _error: PhantomData<fn() -> E>,
    _max_stmts: usize,
    _rng: AtomicCell<ChaCha20>,
    _stream: PhantomData<S>,
    _uri: Secret,
  }

  macro_rules! _executor {
    ($uri_secret:expr, |$config:ident, $uri:ident| $cb:expr) => {{
      $uri_secret
        .peek(&mut Vector::new(), |secret| async move {
          // SAFERT: URI is a string.
          let string = unsafe { core::str::from_utf8_unchecked(&*secret) };
          let $uri = crate::misc::UriRef::new(string);
          let config_rslt = crate::database::client::postgres::Config::from_uri(&$uri);
          let $config = config_rslt?;
          $cb.await
        })?
        .await?
    }};
  }

  impl<E> PostgresRM<E, ()> {
    /// Resource manager for testing purposes.
    #[inline]
    pub fn unit(
      mut rng: ChaCha20,
      secret_context: SecretContext,
      uri: &mut [u8],
    ) -> crate::Result<Self> {
      let _uri = Secret::new(uri, &mut rng, secret_context)?;
      Ok(Self {
        _certs: None,
        _error: PhantomData,
        _max_stmts: DEFAULT_MAX_STMTS,
        _rng: AtomicCell::new(rng),
        _stream: PhantomData,
        _uri,
      })
    }
  }

  impl<E> ResourceManager for PostgresRM<E, ()>
  where
    E: From<crate::Error>,
  {
    type CreateAux = ();
    type Error = E;
    type RecycleAux = ();
    type Resource = PostgresExecutor<E, ExecutorBuffer, ()>;

    #[inline]
    async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
      Ok(_executor!(&self._uri, |config, uri| {
        PostgresExecutor::connect(
          &config,
          ExecutorBuffer::new(self._max_stmts, &mut rng),
          &mut rng,
          (),
        )
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
      let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
      let mut buffer = ExecutorBuffer::new(self._max_stmts, &mut rng);
      mem::swap(&mut buffer, &mut resource.eb);
      *resource = _executor!(&self._uri, |config, uri| {
        PostgresExecutor::connect(&config, buffer, &mut rng, ())
      });
      Ok(())
    }
  }

  #[cfg(feature = "tokio")]
  mod tokio {
    use crate::{
      collection::Vector,
      database::{
        DEFAULT_MAX_STMTS, Executor as _,
        client::postgres::{ExecutorBuffer, PostgresExecutor},
      },
      misc::{Secret, SecretContext},
      pool::{PostgresRM, ResourceManager},
      rng::{ChaCha20, CryptoSeedableRng},
      sync::AtomicCell,
    };
    use core::{marker::PhantomData, mem};
    use tokio::net::TcpStream;

    impl<E> PostgresRM<E, TcpStream> {
      /// Resource manager using the `tokio` project.
      #[inline]
      pub fn tokio(
        mut rng: ChaCha20,
        secret_context: SecretContext,
        uri: &mut [u8],
      ) -> crate::Result<Self> {
        let _uri = Secret::new(uri, &mut rng, secret_context)?;
        Ok(Self {
          _certs: None,
          _error: PhantomData,
          _max_stmts: DEFAULT_MAX_STMTS,
          _rng: AtomicCell::new(rng),
          _stream: PhantomData,
          _uri,
        })
      }
    }

    impl<E> ResourceManager for PostgresRM<E, TcpStream>
    where
      E: From<crate::Error>,
    {
      type CreateAux = ();
      type Error = E;
      type RecycleAux = ();
      type Resource = PostgresExecutor<E, ExecutorBuffer, TcpStream>;

      #[inline]
      async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
        let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
        Ok(_executor!(&self._uri, |config, uri| {
          PostgresExecutor::connect(
            &config,
            ExecutorBuffer::new(self._max_stmts, &mut rng),
            &mut rng,
            TcpStream::connect(uri.hostname_with_implied_port())
              .await
              .map_err(crate::Error::from)?,
          )
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
        let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
        let mut buffer = ExecutorBuffer::new(self._max_stmts, &mut rng);
        mem::swap(&mut buffer, &mut resource.eb);
        *resource = _executor!(&self._uri, |config, uri| {
          PostgresExecutor::connect(
            &config,
            buffer,
            &mut rng,
            TcpStream::connect(uri.hostname_with_implied_port())
              .await
              .map_err(crate::Error::from)?,
          )
        });
        Ok(())
      }
    }
  }

  #[cfg(feature = "tokio-rustls")]
  mod tokio_rustls {
    use crate::{
      collection::Vector,
      database::{
        DEFAULT_MAX_STMTS, Executor as _,
        client::postgres::{ExecutorBuffer, PostgresExecutor},
      },
      misc::{Secret, SecretContext, TokioRustlsConnector},
      pool::{PostgresRM, ResourceManager},
      rng::{ChaCha20, CryptoSeedableRng},
      sync::AtomicCell,
    };
    use core::{marker::PhantomData, mem};
    use tokio::net::TcpStream;
    use tokio_rustls::client::TlsStream;

    impl<E> PostgresRM<E, TlsStream<TcpStream>> {
      /// Resource manager using the `tokio-rustls` project.
      #[inline]
      pub fn tokio_rustls(
        certs: Option<Vector<u8>>,
        mut rng: ChaCha20,
        secret_context: SecretContext,
        uri: &mut [u8],
      ) -> crate::Result<Self> {
        let _uri = Secret::new(uri, &mut rng, secret_context)?;
        Ok(Self {
          _certs: certs,
          _error: PhantomData,
          _max_stmts: DEFAULT_MAX_STMTS,
          _rng: AtomicCell::new(rng),
          _stream: PhantomData,
          _uri,
        })
      }
    }

    impl<E> ResourceManager for PostgresRM<E, TlsStream<TcpStream>>
    where
      E: From<crate::Error>,
    {
      type CreateAux = ();
      type Error = E;
      type RecycleAux = ();
      type Resource = PostgresExecutor<E, ExecutorBuffer, TlsStream<TcpStream>>;

      #[inline]
      async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
        let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
        Ok(_executor!(&self._uri, |config, uri| {
          PostgresExecutor::connect_encrypted(
            &config,
            ExecutorBuffer::new(self._max_stmts, &mut rng),
            &mut rng,
            TcpStream::connect(uri.hostname_with_implied_port())
              .await
              .map_err(crate::Error::from)?,
            |stream| async {
              let mut rslt = TokioRustlsConnector::from_auto()?;
              if let Some(elem) = &self._certs {
                rslt = rslt.push_certs(elem.as_slice())?;
              }
              rslt.connect_without_client_auth(uri.hostname(), stream).await
            },
          )
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
        let mut rng = ChaCha20::from_crypto_rng(&mut &self._rng)?;
        let mut buffer = ExecutorBuffer::new(self._max_stmts, &mut rng);
        mem::swap(&mut buffer, &mut resource.eb);
        *resource = _executor!(&self._uri, |config, uri| {
          PostgresExecutor::connect_encrypted(
            &config,
            buffer,
            &mut rng,
            TcpStream::connect(uri.hostname_with_implied_port())
              .await
              .map_err(crate::Error::from)?,
            |stream| async {
              let mut rslt = TokioRustlsConnector::from_auto()?;
              if let Some(elem) = &self._certs {
                rslt = rslt.push_certs(elem.as_slice())?;
              }
              rslt.connect_without_client_auth(uri.hostname(), stream).await
            },
          )
        });
        Ok(())
      }
    }
  }
}
