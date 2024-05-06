use crate::misc::AsyncBounds;
use core::future::Future;

/// Manager of a specific pool resource.
pub trait ResourceManager {
  /// Auxiliary data used by the [Self::get] method.
  type CreateAux;
  /// Any custom error.
  type Error: From<crate::Error>;
  /// Auxiliary data used by the [Self::recycle] method.
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
  R: AsyncBounds,
  for<'any> &'any Self: AsyncBounds,
{
  type CreateAux = ();
  type Error = E;
  type RecycleAux = ();
  type Resource = R;

  #[inline]
  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    (self.cb)(&self.input)
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
    pub cb: fn(&I) -> CF,
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
      (self.cb)(&self.input).await
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

#[cfg(feature = "http2")]
pub(crate) mod http2 {
  use crate::{
    http2::{Http2Buffer, ReqResBuffer},
    pool::SimpleRM,
    rng::Rng,
  };

  /// Manages HTTP/2 resources for clients.
  pub type Http2ClientBufferRM<RNG> = SimpleRM<crate::Error, RNG, Http2Buffer>;
  /// Manages HTTP/2 resources for servers.
  pub type Http2ServerBufferRM<RNG> = SimpleRM<crate::Error, RNG, Http2Buffer>;
  /// Manages resources for HTTP2 requests and responses.
  pub type ReqResBufferRM = SimpleRM<crate::Error, (), ReqResBuffer>;

  type Http2RM<RNG> = SimpleRM<crate::Error, RNG, Http2Buffer>;

  impl<RNG> Http2RM<RNG>
  where
    RNG: Clone + Rng,
  {
    /// Instance of [Http2ClientRM] or [Http2ServerRM].
    pub fn http2_buffer(rng: RNG) -> Self {
      fn cb<RNG>(rng: &RNG) -> crate::Result<Http2Buffer>
      where
        RNG: Clone + Rng,
      {
        Ok(Http2Buffer::new(rng.clone()))
      }
      Self { cb, input: rng }
    }
  }

  impl ReqResBufferRM {
    /// Instance of [ReqResBufferRM].
    pub fn req_res_buffer() -> Self {
      fn cb(_: &()) -> crate::Result<ReqResBuffer> {
        Ok(ReqResBuffer::default())
      }
      Self { cb, input: () }
    }
  }
}

#[cfg(feature = "web-socket")]
pub(crate) mod web_socket {
  use crate::{
    pool::SimpleRM,
    web_socket::{FrameBufferVec, WebSocketBuffer},
  };

  /// Manages WebSocket resources.
  pub type WebSocketRM = SimpleRM<crate::Error, (), (FrameBufferVec, WebSocketBuffer)>;

  impl WebSocketRM {
    /// Instance of [WebSocketRM].
    pub fn web_socket() -> Self {
      fn cb(_: &()) -> crate::Result<(FrameBufferVec, WebSocketBuffer)> {
        Ok((FrameBufferVec::default(), WebSocketBuffer::default()))
      }
      Self { cb, input: () }
    }
  }
}
