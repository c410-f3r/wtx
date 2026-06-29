use crate::{
  http::{
    AutoStream, ManualStream, OperationMode, StatusCode,
    http2_server_framework::{Http2ServerFrameworkError, ResFinalizer, RouteMatch},
  },
  misc::{FnFut, FnFutWrapper},
};

/// Endpoint that generates a response.
pub trait Endpoint<D, E, S>
where
  E: From<crate::Error>,
{
  /// Operation mode
  const OM: OperationMode = OperationMode::Auto;

  /// Calls endpoint logic of automatic streams
  #[inline]
  fn auto(
    &self,
    _: &mut AutoStream<D>,
    _: (u8, &[RouteMatch]),
  ) -> impl Future<Output = Result<StatusCode, E>> {
    async { Err(crate::Error::from(Http2ServerFrameworkError::OperationModeMismatch).into()) }
  }

  /// Calls endpoint logic of manual streams
  #[inline]
  fn manual(
    &self,
    _: ManualStream<D, S>,
    _: (u8, &[RouteMatch]),
  ) -> impl Future<Output = Result<(), E>> {
    async { Err(crate::Error::from(Http2ServerFrameworkError::OperationModeMismatch).into()) }
  }
}

impl<D, E, S, T> Endpoint<D, E, S> for &T
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
    (*self).auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    (*self).manual(manual_stream, path_defs).await
  }
}

impl<D, E, F, S, RES> Endpoint<D, E, S> for FnFutWrapper<(), F>
where
  E: From<crate::Error>,
  F: FnFut<(), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    auto_stream.req.clear();
    self.0.call(()).await.finalize_response(&mut auto_stream.req)
  }
}

impl<D, E, F, S> Endpoint<D, E, S> for FnFutWrapper<(ManualStream<D, S>,), F>
where
  E: From<crate::Error>,
  F: FnFut<(ManualStream<D, S>,), Result = Result<(), E>>,
{
  const OM: OperationMode = OperationMode::Manual;

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    _: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    self.0.call((manual_stream,)).await?;
    Ok(())
  }
}
