use crate::misc::AsyncBounds;
use core::future::Future;

/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Auxiliary data used by the [`Self::get`] method.
  type CreateAux;
  /// Any custom error.
  type Error: From<crate::Error>;
  /// Auxiliary data used by the [`Self::recycle`] method.
  type RecycleAux;
  /// Any pool resource.
  type Resource;

  /// Creates a new resource instance based on the contents of this manager.
  fn create(
    &self,
    aux: &Self::CreateAux,
  ) -> impl AsyncBounds + Future<Output = Result<Self::Resource, Self::Error>>;

  /// If a resource is in an invalid state.
  fn is_invalid(&self, resource: &Self::Resource) -> bool;

  /// Re-creates a new valid instance. Should be called if `resource` is invalid.
  fn recycle(
    &self,
    aux: &Self::RecycleAux,
    resource: &mut Self::Resource,
  ) -> impl AsyncBounds + Future<Output = Result<(), Self::Error>>;
}

impl ResourceManager for () {
  type CreateAux = ();
  type Error = crate::Error;
  type RecycleAux = ();
  type Resource = ();

  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Ok(())
  }

  fn is_invalid(&self, _: &Self::Resource) -> bool {
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
  R: AsyncBounds,
  for<'any> &'any Self: AsyncBounds,
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

#[cfg(feature = "postgres")]
pub(crate) mod database {
  use crate::{
    database::client::postgres::{Executor, ExecutorBuffer},
    misc::AsyncBounds,
    pool::ResourceManager,
  };
  use core::{future::Future, mem};

  /// Manages generic database executors.
  #[derive(Debug)]
  pub struct PostgresRM<CF, I, RF> {
    /// Create callback
    pub cc: fn(&I) -> CF,
    /// Input
    pub input: I,
    /// Recycle callback
    pub rc: fn(&I, ExecutorBuffer) -> RF,
  }

  impl<CF, I, E, O, RF, S> ResourceManager for PostgresRM<CF, I, RF>
  where
    CF: AsyncBounds + Future<Output = Result<(O, Executor<E, ExecutorBuffer, S>), E>>,
    E: AsyncBounds + From<crate::Error>,
    O: AsyncBounds,
    RF: AsyncBounds + Future<Output = Result<Executor<E, ExecutorBuffer, S>, E>>,
    S: AsyncBounds,
    for<'any> &'any Self: AsyncBounds,
  {
    type CreateAux = ();
    type Error = E;
    type RecycleAux = ();
    type Resource = (O, Executor<E, ExecutorBuffer, S>);

    #[inline]
    async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      (self.cc)(&self.input).await
    }

    #[inline]
    fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.1.is_closed
    }

    #[inline]
    async fn recycle(
      &self,
      _: &Self::RecycleAux,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let mut persistent = ExecutorBuffer::_empty();
      mem::swap(&mut persistent, &mut resource.1.eb);
      resource.1 = (self.rc)(&self.input, persistent).await?;
      Ok(())
    }
  }
}
