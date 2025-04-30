#![cfg_attr(feature = "_bench", allow(soft_unstable))]
#![cfg_attr(feature = "_bench", feature(test))]
#![cfg_attr(feature = "nightly", feature(return_type_notation))]
#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;
#[allow(unused_extern_crates, reason = "selection of features")]
#[cfg(all(feature = "_bench", test))]
extern crate test;

#[macro_use]
mod macros;

#[cfg(all(feature = "_bench", test))]
pub(crate) mod bench;
#[cfg(feature = "client-api-framework")]
pub mod client_api_framework;
pub mod collection;
#[cfg(feature = "data-transformation")]
pub mod data_transformation;
#[cfg(feature = "database")]
pub mod database;
mod error;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http2")]
pub mod http2;
pub mod misc;
#[cfg(feature = "pool")]
pub mod pool;
pub mod rng;
pub mod stream;
pub mod sync;
#[cfg(test)]
mod tests;
pub mod time;
#[cfg(feature = "web-socket")]
pub mod web_socket;

pub use error::Error;
#[cfg(feature = "std")]
pub use error::VarError;

pub(crate) const _MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;

/// Shortcut of [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;
