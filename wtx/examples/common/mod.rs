#![allow(unused_imports)]

#[cfg(feature = "web-socket")]
mod web_socket;

#[cfg(feature = "web-socket")]
pub(crate) use web_socket::{_accept_conn_and_echo_frames, _handle_frames};

pub(crate) fn _host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_owned())
}

pub(crate) fn _uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:8080".to_owned())
}
