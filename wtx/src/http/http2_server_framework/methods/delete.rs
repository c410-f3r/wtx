use crate::{
  collections::{ArrayVectorCopy, Vector},
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, RouteMatch, methods::check_method},
  },
  misc::FnFut,
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
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    check_method(Method::Delete, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Delete, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Delete<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVectorCopy<RouteMatch, 4>,
    _: &mut Vector<ArrayVectorCopy<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
