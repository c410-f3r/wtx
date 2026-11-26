use crate::{
  collections::{ArrayVectorCopy, VectorU16},
  futures::FnFut,
  http::{
    AutoStream, ManualStream, Method, Mime, OperationMode, StatusCode,
    http2_server_framework::{
      Endpoint, EndpointNode, HttpRouterNode, misc::check_header_and_method,
    },
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
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<StatusCode, E> {
    let _ = check_header_and_method(
      Mime::ApplicationJson,
      &auto_stream.req.msg_data.headers,
      auto_stream.req.method,
      self.1,
    )?;
    self.0.auto(auto_stream, path_leafs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<(), E> {
    let _ = check_header_and_method(
      Mime::ApplicationJson,
      &manual_stream.req.msg_data.headers,
      manual_stream.req.method,
      self.1,
    )?;
    self.0.manual(manual_stream, path_leafs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for Json<T>
where
  E: From<crate::Error>,
  T: Endpoint<D, E, S>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn method(&self) -> Option<Method> {
    Some(self.1)
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
