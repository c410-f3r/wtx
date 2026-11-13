use crate::{
  collections::{ArrayVectorCopy, VectorUsize},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, RouteMatch},
  },
};

/// Requires a request of type `DELETE`.
#[derive(Clone, Debug)]
pub struct Any<T>(
  /// Function
  pub T,
);

/// Creates a new [`Any`] instance.
#[inline]
pub fn any<A, T>(ty: T) -> Any<T::Wrapper>
where
  T: FnFut<A>,
{
  Any(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for Any<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const OM: OperationMode = T::OM;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Any<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVectorCopy<RouteMatch, 4>,
    _: &mut VectorUsize<ArrayVectorCopy<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
