use crate::data_transformation::format::JsonRpcResponseError;
use alloc::boxed::Box;

/// Client API Framework Error
#[derive(Debug)]
pub enum DataTransformationError {
  /// JSON-RPC response error
  JsonRpcResultErr(Box<JsonRpcResponseError>),
  /// `wtx` can not perform this operation due to known limitations.
  UnsupportedOperation,
}
