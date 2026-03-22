use crate::codec::protocol::JsonRpcResponseError;
use alloc::boxed::Box;

/// Decode/EnCode error
#[derive(Debug)]
pub enum CodecError {
  /// JSON-RPC response error
  JsonRpcDecoderErr(Box<JsonRpcResponseError>),
  /// `wtx` can not perform this operation due to known limitations.
  UnsupportedOperation,
}
