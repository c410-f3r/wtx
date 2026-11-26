use crate::{
  collections::{ArrayVectorCopy, VectorU16},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, HttpRouterNode, misc::check_method},
  },
};

/// Requires a request of type `POST`.
#[derive(Clone, Debug)]
pub struct Post<T>(
  /// Function
  pub T,
);

/// Creates a new [`Post`] instance.
#[inline]
pub fn post<A, T>(ty: T) -> Post<T::Wrapper>
where
  T: FnFut<A>,
{
  Post(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for Post<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const OM: OperationMode = T::OM;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<StatusCode, E> {
    check_method(Method::Post, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_leafs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<(), E> {
    check_method(Method::Post, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_leafs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Post<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn method(&self) -> Option<Method> {
    Some(Method::Post)
  }

  #[inline]
  fn paths_indices(
    &self,
    _: &mut VectorU16<ArrayVectorCopy<HttpRouterNode, 4>>,
    _: ArrayVectorCopy<HttpRouterNode, 4>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
