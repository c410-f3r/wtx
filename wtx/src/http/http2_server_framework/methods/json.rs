use crate::{
  collections::{ArrayVectorCopy, Vector},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, Mime, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, RouteMatch, misc::check_header_and_method},
  },
};

/// Requires a JSON request as well as an associated method.
#[derive(Clone, Debug)]
pub struct Json<T>(
  /// Function
  pub T,
  /// Required method
  pub Method,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn json<A, T>(method: Method, ty: T) -> Json<T::Wrapper>
where
  T: FnFut<A>,
{
  Json(ty.into_wrapper(), method)
}

impl<D, E, S, T> Endpoint<D, E, S> for Json<T>
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
    let _ = check_header_and_method(
      Mime::ApplicationJson,
      &auto_stream.req.msg_data.headers,
      auto_stream.req.method,
      self.1,
    )?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    let _ = check_header_and_method(
      Mime::ApplicationJson,
      &manual_stream.req.msg_data.headers,
      manual_stream.req.method,
      self.1,
    )?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Json<T>
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
