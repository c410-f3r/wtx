//! gRPC (gRPC Remote Procedure Calls[2]) is a high performance remote procedure call (RPC)
//! framework.

mod client;
mod grpc_status_code;
mod server;
mod server_data;

use crate::{data_transformation::dnsn::Serialize, misc::Vector};
pub use client::Client;
pub use grpc_status_code::GrpcStatusCode;
pub use server::Server;
pub use server_data::ServerData;

#[inline]
fn serialize<DRSR, T>(bytes: &mut Vector<u8>, mut data: T, drsr: &mut DRSR) -> crate::Result<()>
where
  T: Serialize<DRSR>,
{
  bytes.extend_from_slice(&[0; 5])?;
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
