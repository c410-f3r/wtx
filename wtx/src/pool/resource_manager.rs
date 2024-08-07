/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Auxiliary data used by the [`Self::get`] method.
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
  fn is_invalid(&self, resource: &Self::Resource) -> impl Future<Output = bool>;

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

  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Ok(())
  }

  async fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

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
  pub fn new(cb: F) -> Self {
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
  async fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  #[inline]
  async fn recycle(&self, _: &Self::RecycleAux, _: &mut Self::Resource) -> Result<(), Self::Error> {
    Ok(())
  }
}

#[cfg(feature = "postgres")]
pub(crate) mod database {
  use crate::rng::StdRngSync;
  use core::marker::PhantomData;

  /// Manages generic database executors.
  #[derive(Debug)]
  pub struct PostgresRM<E, S> {
    _certs: Option<&'static [u8]>,
    error: PhantomData<fn() -> E>,
    rng: StdRngSync,
    stream: PhantomData<S>,
    uri: &'static str,
  }

  macro_rules! executor {
    ($uri_str:expr, |$config:ident, $uri:ident| $cb:expr) => {{
      let $uri = crate::misc::UriRef::new($uri_str);
      let config_rslt = crate::database::client::postgres::Config::from_uri(&$uri);
      let $config = config_rslt.map_err(Into::into)?;
      $cb.await.map_err(Into::into)
    }};
  }

  #[cfg(feature = "tokio")]
  mod tokio {
    use crate::{
      database::{
        client::postgres::{Executor, ExecutorBuffer},
        Executor as _,
      },
      pool::{PostgresRM, ResourceManager},
      rng::StdRngSync,
    };
    use core::mem;
    use std::marker::PhantomData;
    use tokio::net::TcpStream;

    impl<E> PostgresRM<E, TcpStream> {
      /// Resource manager using the `tokio` project.
      #[inline]
      pub fn tokio(uri: &'static str) -> Self {
        Self {
          _certs: None,
          error: PhantomData,
          rng: StdRngSync::default(),
          stream: PhantomData,
          uri,
        }
      }
    }

    impl<E> ResourceManager for PostgresRM<E, TcpStream>
    where
      E: From<crate::Error>,
    {
      type CreateAux = ();
      type Error = E;
      type RecycleAux = ();
      type Resource = Executor<E, ExecutorBuffer, TcpStream>;

      #[inline]
      async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
        executor!(self.uri, |config, uri| {
          let eb = ExecutorBuffer::with_default_params(&mut &self.rng)?;
          let stream = TcpStream::connect(uri.host()).await.map_err(Into::into)?;
          Executor::connect(&config, eb, &mut &self.rng, stream)
        })
      }

      #[inline]
      async fn is_invalid(&self, resource: &Self::Resource) -> bool {
        resource.connection_state().is_closed()
      }

      #[inline]
      async fn recycle(
        &self,
        _: &Self::RecycleAux,
        resource: &mut Self::Resource,
      ) -> Result<(), Self::Error> {
        let mut buffer = ExecutorBuffer::_empty();
        mem::swap(&mut buffer, &mut resource.eb);
        *resource = executor!(self.uri, |config, uri| {
          let stream = TcpStream::connect(uri.host()).await.map_err(Into::into)?;
          Executor::connect(&config, buffer, &mut &self.rng, stream)
        })?;
        Ok(())
      }
    }
  }

  #[cfg(feature = "tokio-rustls")]
  mod tokio_rustls {
    use crate::{
      database::{
        client::postgres::{Executor, ExecutorBuffer},
        Executor as _,
      },
      misc::TokioRustlsConnector,
      pool::{PostgresRM, ResourceManager},
      rng::StdRngSync,
    };
    use core::mem;
    use std::marker::PhantomData;
    use tokio::net::TcpStream;
    use tokio_rustls::client::TlsStream;

    impl<E> PostgresRM<E, TlsStream<TcpStream>> {
      /// Resource manager using the `tokio-rustls` project.
      #[inline]
      pub fn tokio_rustls(certs: Option<&'static [u8]>, uri: &'static str) -> Self {
        Self {
          _certs: certs,
          error: PhantomData,
          rng: StdRngSync::default(),
          stream: PhantomData,
          uri,
        }
      }
    }

    impl<E> ResourceManager for PostgresRM<E, TlsStream<TcpStream>>
    where
      E: From<crate::Error>,
    {
      type CreateAux = ();
      type Error = E;
      type RecycleAux = ();
      type Resource = Executor<E, ExecutorBuffer, TlsStream<TcpStream>>;

      #[inline]
      async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
        executor!(self.uri, |config, uri| {
          Executor::connect_encrypted(
            &config,
            ExecutorBuffer::with_default_params(&mut &self.rng)?,
            TcpStream::connect(uri.host()).await.map_err(Into::into)?,
            &mut &self.rng,
            |stream| async {
              #[allow(unused_mut, reason = "features")]
              let mut rslt = TokioRustlsConnector::from_webpki_roots();
              #[cfg(feature = "rustls-pemfile")]
              if let Some(elem) = self._certs {
                rslt = rslt.push_certs(elem)?;
              }
              Ok(rslt.with_generic_stream(uri.hostname(), stream).await?)
            },
          )
        })
      }

      #[inline]
      async fn is_invalid(&self, resource: &Self::Resource) -> bool {
        resource.connection_state().is_closed()
      }

      #[inline]
      async fn recycle(
        &self,
        _: &Self::RecycleAux,
        resource: &mut Self::Resource,
      ) -> Result<(), Self::Error> {
        let mut buffer = ExecutorBuffer::_empty();
        mem::swap(&mut buffer, &mut resource.eb);
        *resource = executor!(self.uri, |config, uri| {
          Executor::connect_encrypted(
            &config,
            ExecutorBuffer::with_default_params(&mut &self.rng)?,
            TcpStream::connect(uri.host()).await.map_err(Into::into)?,
            &mut &self.rng,
            |stream| async {
              #[allow(unused_mut, reason = "features")]
              let mut rslt = TokioRustlsConnector::from_webpki_roots();
              #[cfg(feature = "rustls-pemfile")]
              if let Some(elem) = self._certs {
                rslt = rslt.push_certs(elem)?;
              }
              Ok(rslt.with_generic_stream(uri.hostname(), stream).await?)
            },
          )
        })?;
        Ok(())
      }
    }
  }
}
