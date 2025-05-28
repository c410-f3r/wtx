use crate::{
  http::{
    AutoStream, ManualStream, OperationMode, StatusCode,
    server_framework::{ResFinalizer, RouteMatch, ServerFrameworkError},
  },
  misc::{FnFut, FnFutWrapper},
};

/// Endpoint that generates a response.
pub trait Endpoint<CA, E, S, SA>
where
  E: From<crate::Error>,
{
  /// Operation mode
  const OM: OperationMode = OperationMode::Auto;

  /// Calls endpoint logic of automatic streams
  fn auto(
    &self,
    _: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> impl Future<Output = Result<StatusCode, E>> {
    async { Err(crate::Error::from(ServerFrameworkError::OperationModeMismatch).into()) }
  }

  /// Calls endpoint logic of manual streams
  fn manual(
    &self,
    _: ManualStream<CA, S, SA>,
    _: (u8, &[RouteMatch]),
  ) -> impl Future<Output = Result<(), E>> {
    async { Err(crate::Error::from(ServerFrameworkError::OperationModeMismatch).into()) }
  }
}

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for &T
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
    (*self).auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    (*self).manual(manual_stream, path_defs).await
  }
}

impl<CA, E, F, S, SA, RES> Endpoint<CA, E, S, SA> for FnFutWrapper<(), F>
where
  E: From<crate::Error>,
  F: FnFut<(), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    auto_stream.req.clear();
    self.0.call(()).await.finalize_response(&mut auto_stream.req)
  }
}

impl<CA, E, F, S, SA> Endpoint<CA, E, S, SA> for FnFutWrapper<(ManualStream<CA, S, SA>,), F>
where
  E: From<crate::Error>,
  F: FnFut<(ManualStream<CA, S, SA>,), Result = Result<(), E>>,
{
  const OM: OperationMode = OperationMode::Manual;

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    self.0.call((manual_stream,)).await?;
    Ok(())
  }
}
