use crate::{
  collection::{ArrayVectorU8, Vector},
  http::{
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
    server_framework::{Endpoint, EndpointNode, RouteMatch, methods::check_method},
  },
  misc::FnFut,
};

/// Requires a request of type `PATCH`.
#[derive(Debug)]
pub struct Patch<T>(
  /// Function
  pub T,
);

/// Creates a new [`Patch`] instance.
#[inline]
pub fn patch<A, T>(ty: T) -> Patch<T::Wrapper>
where
  T: FnFut<A>,
{
  Patch(ty.into_wrapper())
}

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for Patch<T>
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
    check_method(Method::Patch, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Patch, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for Patch<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
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
