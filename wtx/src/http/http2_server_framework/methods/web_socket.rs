use crate::{
  collections::{ArrayVectorCopy, Vector},
  http::{
    AutoStream, ManualStream, OperationMode, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, Http2ServerFrameworkError, RouteMatch},
    is_web_socket_handshake,
  },
  misc::FnFut,
};

/// Requires a WebSocket tunneling.
#[derive(Clone, Debug)]
pub struct WebSocket<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`WebSocket`] instance.
#[inline]
pub fn web_socket<A, T>(ty: T) -> WebSocket<T::Wrapper>
where
  T: FnFut<A>,
{
  WebSocket(ty.into_wrapper())
}

impl<D, E, S, T> Endpoint<D, E, S> for WebSocket<T>
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
    if !is_web_socket_handshake(
      &auto_stream.req.msg_data.headers,
      auto_stream.req.method,
      auto_stream.protocol,
    ) {
      return Err(crate::Error::from(Http2ServerFrameworkError::InvalidWebSocketParameters).into());
    }
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<D, S>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    if !is_web_socket_handshake(
      &manual_stream.req.msg_data.headers,
      manual_stream.req.method,
      manual_stream.protocol,
    ) {
      return Err(crate::Error::from(Http2ServerFrameworkError::InvalidWebSocketParameters).into());
    }
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<D, E, S, T> EndpointNode<D, E, S> for WebSocket<T>
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
