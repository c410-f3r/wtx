use core::future::Future;

/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Any custom error.
  type Error;
  /// Any pool resource.
  type Resource;

  /// Creates a new resource instance based on the contents of this manager.
  fn create(&self) -> impl Future<Output = Result<Self::Resource, Self::Error>>;

  /// If a resource is in an invalid state.
  fn is_invalid(&self, resource: &Self::Resource) -> bool;

  /// Re-creates a new valid instance. Should be called if `resource` is invalid.
  fn recycle(&self, resource: &mut Self::Resource)
    -> impl Future<Output = Result<(), Self::Error>>;
}

/// Manages generic resources that are always valid and don't require logic for recycling.
#[derive(Debug)]
pub struct SimpleRM<E, I, R> {
  /// Create callback
  pub cb: fn(&I) -> Result<R, E>,
  /// Input data
  pub input: I,
}

impl<E, I, R> SimpleRM<E, I, R> {
  /// Shortcut constructor
  #[inline]
  pub fn new(cb: fn(&I) -> Result<R, E>, input: I) -> Self {
    Self { cb, input }
  }
}

impl<E, I, R> ResourceManager for SimpleRM<E, I, R>
where
  E: From<crate::Error>,
{
  type Error = E;
  type Resource = R;

  #[inline]
  async fn create(&self) -> Result<Self::Resource, Self::Error> {
    (self.cb)(&self.input)
  }

  #[inline]
  fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  #[inline]
  async fn recycle(&self, _: &mut Self::Resource) -> Result<(), Self::Error> {
    Ok(())
  }
}

#[cfg(feature = "postgres")]
pub(crate) mod database {
  use crate::{
    database::client::postgres::{Executor, ExecutorBuffer},
    pool_manager::ResourceManager,
  };
  use core::{future::Future, mem};

  /// Manages generic database executors.
  #[derive(Debug)]
  pub struct PostgresRM<CF, I, RF> {
    /// Create callback
    pub cb: fn(&I) -> CF,
    /// Input
    pub input: I,
    /// Recycle callback
    pub rc: fn(&I, ExecutorBuffer) -> RF,
  }

  impl<CF, I, E, O, RF, S> ResourceManager for PostgresRM<CF, I, RF>
  where
    CF: Future<Output = Result<(O, Executor<E, ExecutorBuffer, S>), E>>,
    E: From<crate::Error>,
    RF: Future<Output = Result<Executor<E, ExecutorBuffer, S>, E>>,
  {
    type Error = E;
    type Resource = (O, Executor<E, ExecutorBuffer, S>);

    #[inline]
    async fn create(&self) -> Result<Self::Resource, Self::Error> {
      (self.cb)(&self.input).await
    }

    #[inline]
    fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.1.is_closed
    }

    #[inline]
    async fn recycle(&self, resource: &mut Self::Resource) -> Result<(), Self::Error> {
      let mut persistent = ExecutorBuffer::_empty();
      mem::swap(&mut persistent, &mut resource.1.eb);
      resource.1 = (self.rc)(&self.input, persistent).await?;
      Ok(())
    }
  }
}

#[cfg(feature = "web-socket")]
pub(crate) mod web_socket {
  use crate::{
    pool_manager::SimpleRM,
    web_socket::{FrameBufferVec, WebSocketBuffer},
  };

  /// Manages WebSocket resources.
  pub type WebSocketRM = SimpleRM<crate::Error, (), (FrameBufferVec, WebSocketBuffer)>;

  impl WebSocketRM {
    /// Instance of [WebSocketRM].
    pub fn web_socket_rm() -> WebSocketRM {
      fn cb(_: &()) -> crate::Result<(FrameBufferVec, WebSocketBuffer)> {
        Ok(<_>::default())
      }
      Self { cb, input: () }
    }
  }
}
