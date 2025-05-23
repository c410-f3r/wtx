use crate::{
  collection::{ArrayVector, Vector},
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    server_framework::{Endpoint, EndpointNode, RouteMatch, methods::check_method},
  },
  misc::FnFut,
};

/// Requires a request of type `PUT`.
#[derive(Debug)]
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

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for Put<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const OM: OperationMode = T::OM;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    check_method(Method::Put, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Put, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for Put<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<RouteMatch, 4>,
    _: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
