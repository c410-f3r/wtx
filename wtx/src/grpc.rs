//! gRPC (gRPC Remote Procedure Calls) is a high performance remote procedure call (RPC)
//! framework.

mod client;
mod grpc_manager;
mod grpc_res_middleware;
mod grpc_status_code;

use crate::{data_transformation::dnsn::Serialize, misc::Vector};
pub use client::Client;
pub use grpc_manager::GrpcManager;
pub use grpc_res_middleware::GrpcMiddleware;
pub use grpc_status_code::GrpcStatusCode;

#[inline]
fn serialize<DRSR, T>(bytes: &mut Vector<u8>, mut data: T, drsr: &mut DRSR) -> crate::Result<()>
where
  T: Serialize<DRSR>,
{
  bytes.extend_from_copyable_slice(&[0; 5])?;
  let before_len = bytes.len();
  data.to_bytes(bytes, drsr)?;
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
