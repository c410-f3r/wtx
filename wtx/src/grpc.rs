//! gRPC (gRPC Remote Procedure Calls) is a high performance remote procedure call (RPC)
//! framework.

#[cfg(feature = "grpc-client")]
mod grpc_client;
mod grpc_manager;
#[cfg(feature = "grpc-server")]
mod grpc_middleware;
mod grpc_status_code;

use crate::{
  codec::{Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
};

#[cfg(feature = "grpc-client")]
pub use grpc_client::GrpcClient;
pub use grpc_manager::GrpcManager;
#[cfg(feature = "grpc-server")]
pub use grpc_middleware::GrpcMiddleware;
pub use grpc_status_code::GrpcStatusCode;

fn serialize<'drsr, DRSR, T>(
  bytes: &mut Vector<u8>,
  data: T,
  drsr: &'drsr mut DRSR,
) -> crate::Result<()>
where
  T: Encode<GenericCodec<&'drsr mut DRSR, &'drsr mut DRSR>>,
{
  bytes.extend_from_copyable_slice(&[0; 5])?;
  let before_len = bytes.len();
  data.encode(&mut EncodeWrapper::new(bytes, drsr))?;
  let after_len = bytes.len();
  if let [_, a, b, c, d, ..] = bytes.as_mut() {
    let len = u32::try_from(after_len.wrapping_sub(before_len)).unwrap_or_default();
    let [e, f, g, h] = len.to_be_bytes();
    *a = e;
    *b = f;
    *c = g;
    *d = h;
  }
  Ok(())
}
