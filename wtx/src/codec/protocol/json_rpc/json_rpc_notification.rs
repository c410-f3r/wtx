use crate::{codec::protocol::JsonRpcNotificationParams, collection::ArrayStringU8};

/// A request object without an "id" member. Generally used with WebSocket connections.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct JsonRpcNotification<R> {
  /// Name of the method invoked.
  pub method: Option<ArrayStringU8<31>>,
  /// See [`JsonRpcNotificationParams`].
  pub params: JsonRpcNotificationParams<R>,
}
