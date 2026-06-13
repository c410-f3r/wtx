use crate::{
  collection::{ArrayVectorU8, Vector},
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, RouteMatch, methods::check_method},
  },
  misc::FnFut,
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
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    check_method(Method::Put, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Put, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Put<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVectorU8<RouteMatch, 4>,
    _: &mut Vector<ArrayVectorU8<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
