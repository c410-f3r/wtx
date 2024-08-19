use alloc::boxed::Box;

/// When a rpc call encounters an error.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct JsonRpcResponseError {
  /// Indicates the error type that occurred.
  pub code: i32,
  /// Additional information about the error
  #[cfg(feature = "serde_json")]
  pub data: Option<serde_json::Value>,
  /// Short description of the error.
  pub message: Box<str>,
}
