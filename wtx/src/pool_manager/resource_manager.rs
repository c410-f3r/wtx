use core::future::Future;

/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Any custom error.
  type Error;
  /// Subset of a resource's data used across different instances.
  type Persistent;
  /// Any pool resource.
  type Resource;

  /// If a resource is in an invalid state, then returns its persistent content to attempt to create
  /// a new valid instance.
  fn check_integrity(&self, resource: &mut Self::Resource) -> Option<Self::Persistent>;

  /// Creates a new resource instance based on the contents of this manager.
  fn create(&self) -> impl Future<Output = Result<Self::Resource, Self::Error>>;

  /// Re-creates a new valid instance using the persistent data of an invalid instance.
  fn recycle(
    &self,
    persistent: Self::Persistent,
    resource: &mut Self::Resource,
  ) -> impl Future<Output = Result<(), Self::Error>>;
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
  pub struct PostgresRM<CF, I, RF>(
    /// Create callback
    pub fn(&I) -> CF,
    /// Input data
    pub I,
    /// Recycle callback
    pub fn(&I, ExecutorBuffer) -> RF,
  );

  impl<C, I, ERR, O, RF, S> ResourceManager for PostgresRM<C, I, RF>
  where
    ERR: From<crate::Error>,
    C: Future<Output = Result<(O, Executor<ERR, ExecutorBuffer, S>), ERR>>,
    RF: Future<Output = Result<Executor<ERR, ExecutorBuffer, S>, ERR>>,
  {
    type Error = ERR;
    type Persistent = ExecutorBuffer;
    type Resource = (O, Executor<ERR, ExecutorBuffer, S>);

    #[inline]
    async fn create(&self) -> Result<Self::Resource, Self::Error> {
      (self.0)(&self.1).await
    }

    #[inline]
    fn check_integrity(&self, resource: &mut Self::Resource) -> Option<Self::Persistent> {
      if resource.1.is_closed {
        let mut rslt = ExecutorBuffer::_empty();
        mem::swap(&mut rslt, &mut resource.1.eb);
        Some(rslt)
      } else {
        None
      }
    }

    #[inline]
    async fn recycle(
      &self,
      persistent: Self::Persistent,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let executor = (self.2)(&self.1, persistent).await?;
      resource.1 = executor;
      Ok(())
    }
  }
}

#[cfg(feature = "web-socket")]
pub(crate) mod websocket {
  use crate::{
    pool_manager::resource_manager::ResourceManager,
    web_socket::{FrameBufferVec, WebSocketBuffer},
  };

  /// Manages WebSocket resources.
  #[derive(Debug)]
  pub struct WebSocketRM;

  impl ResourceManager for WebSocketRM {
    type Error = crate::Error;
    type Persistent = ();
    type Resource = (FrameBufferVec, WebSocketBuffer);

    #[inline]
    async fn create(&self) -> Result<Self::Resource, Self::Error> {
      Ok(<_>::default())
    }

    #[inline]
    fn check_integrity(&self, _: &mut Self::Resource) -> Option<Self::Persistent> {
      None
    }

    #[inline]
    async fn recycle(
      &self,
      _: Self::Persistent,
      _: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      Ok(())
    }
  }
}
