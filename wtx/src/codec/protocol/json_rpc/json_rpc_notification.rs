use crate::{codec::protocol::JsonRpcNotificationParams, collections::ArrayStrU8};

/// A request object without an "id" member. Generally used with WebSocket connections.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct JsonRpcNotification<R> {
  /// Name of the method invoked.
  pub method: Option<ArrayStrU8<31>>,
  /// See [`JsonRpcNotificationParams`].
  pub params: JsonRpcNotificationParams<R>,
}
