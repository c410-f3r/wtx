use crate::data_transformation::format::JsonRpcNotificationParams;
use alloc::boxed::Box;

/// A request object without an "id" member. Generally used with WebSocket connections.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct JsonRpcNotification<R> {
  /// Name of the method invoked.
  pub method: Option<Box<str>>,
  /// See [`JsonRpcNotificationParams`].
  pub params: JsonRpcNotificationParams<R>,
}
