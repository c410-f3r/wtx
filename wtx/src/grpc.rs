//! gRPC (gRPC Remote Procedure Calls[2]) is a high performance remote procedure call (RPC)
//! framework.

mod client;
mod server;

pub use client::Client;
pub use server::Server;
