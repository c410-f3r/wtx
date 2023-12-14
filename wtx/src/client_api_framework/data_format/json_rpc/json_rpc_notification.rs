use crate::client_api_framework::data_format::JsonRpcNotificationParams;
use alloc::string::String;

/// A request object without an "id" member. Generally used with WebSocket connections.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct JsonRpcNotification<R> {
  /// Name of the method invoked.
  pub method: Option<String>,
  /// See [JsonRpcNotificationParams].
  pub params: JsonRpcNotificationParams<R>,
}
