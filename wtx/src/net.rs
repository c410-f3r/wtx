//! Networking primitives

#[macro_use]
mod macros;

mod buf_stream_reader;
mod bytes_stream;
mod connection_state;
#[cfg(feature = "embassy-net")]
mod embassy_net;
mod misc;
mod net_error;
mod role;
#[cfg(feature = "std")]
mod std;
mod stream;
mod stream_common;
mod stream_reader;
mod stream_writer;
mod tcp_listener;
mod tcp_params;
mod tcp_stream;
mod to_socket_addrs;
#[cfg(feature = "tokio")]
mod tokio;
mod udp_stream;
mod uri;

pub use buf_stream_reader::BufStreamReader;
pub use bytes_stream::BytesStream;
pub use connection_state::ConnectionState;
pub use net_error::NetError;
pub use role::{Client, Role, RoleTy, Server};
pub use stream::Stream;
pub use stream_common::StreamCommon;
pub use stream_reader::StreamReader;
pub use stream_writer::StreamWriter;
pub use tcp_listener::TcpListener;
pub use tcp_params::TcpParams;
pub use tcp_stream::TcpStream;
pub use to_socket_addrs::ToSocketAddrs;
pub use udp_stream::UdpStream;
pub use uri::{QueryWriter, Uri, UriArrayString, UriBox, UriCow, UriRef, UriReset, UriString};
