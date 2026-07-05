use crate::{
  collections::{ArrayVectorCopy, Vector},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, RouteMatch, methods::check_method},
  },
};

/// Requires a request of type `GET`.
#[derive(Clone, Debug)]
pub struct Get<T>(
  /// Function
  pub T,
);

/// Creates a new [`Get`] instance.
#[inline]
pub fn get<A, T>(ty: T) -> Get<T::Wrapper>
where
  T: FnFut<A>,
{
  Get(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for Get<T>
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
    check_method(Method::Get, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Get, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Get<T>
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
