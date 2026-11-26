use crate::{
  collections::{ArrayVectorCopy, VectorU16},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, HttpRouterNode, misc::check_method},
  },
};

/// Requires a request of type `PUT`.
#[derive(Clone, Debug)]
pub struct Put<T>(
  /// Function
  pub T,
);

/// Creates a new [`Put`] instance.
#[inline]
pub fn put<A, T>(ty: T) -> Put<T::Wrapper>
where
  T: FnFut<A>,
{
  Put(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for Put<T>
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
    check_method(Method::Put, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_leafs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<(), E> {
    check_method(Method::Put, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_leafs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Put<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn method(&self) -> Option<Method> {
    Some(Method::Put)
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
