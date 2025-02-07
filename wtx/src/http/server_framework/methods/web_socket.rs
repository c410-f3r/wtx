use crate::{
  http::{
    AutoStream, ManualStream, OperationMode, StatusCode, is_web_socket_handshake,
    server_framework::{Endpoint, EndpointNode, RouteMatch, ServerFrameworkError},
  },
  misc::{ArrayVector, FnFut, Vector},
};

/// Requires a WebSocket tunneling.
#[derive(Debug)]
pub struct WebSocket<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn web_socket<A, T>(ty: T) -> WebSocket<T::Wrapper>
where
  T: FnFut<A>,
{
  WebSocket(ty.into_wrapper())
}

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for WebSocket<T>
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
    if !is_web_socket_handshake(
      &auto_stream.req.rrd.headers,
      auto_stream.req.method,
      auto_stream.protocol,
    ) {
      return Err(crate::Error::from(ServerFrameworkError::InvalidWebSocketParameters).into());
    }
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    if !is_web_socket_handshake(
      &manual_stream.req.rrd.headers,
      manual_stream.req.method,
      manual_stream.protocol,
    ) {
      return Err(crate::Error::from(ServerFrameworkError::InvalidWebSocketParameters).into());
    }
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for WebSocket<T>
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
