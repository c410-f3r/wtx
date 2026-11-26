use crate::{
  collections::{ArrayVectorCopy, VectorU16},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, HttpRouterNode, misc::check_method},
  },
};

/// Requires a request of type `DELETE`.
#[derive(Clone, Debug)]
pub struct Delete<T>(
  /// Function
  pub T,
);

/// Creates a new [`Delete`] instance.
#[inline]
pub fn delete<A, T>(ty: T) -> Delete<T::Wrapper>
where
  T: FnFut<A>,
{
  Delete(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for Delete<T>
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
    check_method(Method::Delete, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_leafs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<(), E> {
    check_method(Method::Delete, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_leafs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Delete<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn method(&self) -> Option<Method> {
    Some(Method::Delete)
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
