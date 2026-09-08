#![allow(unused_features, reason = "remove this once the features are stabilized")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "_bench", feature(test))]
#![doc = include_str!("../README.md")]
#![feature(random, return_type_notation)]
#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate self as wtx;
#[cfg(feature = "std")]
extern crate std;
#[allow(unused_extern_crates, reason = "selection of features")]
#[cfg(all(feature = "_bench", test))]
extern crate test;

#[macro_use]
mod macros;

#[cfg(feature = "asn1")]
pub mod asn1;
#[cfg(all(feature = "_bench", test))]
pub(crate) mod bench;
pub mod calendar;
#[cfg(feature = "client-api-framework")]
pub mod client_api_framework;
pub mod codec;
pub mod collections;
#[cfg(feature = "crypto")]
pub mod crypto;
#[cfg(feature = "database")]
pub mod database;
mod error;
pub mod executor;
pub mod futures;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http2")]
pub mod http2;
pub mod misc;
pub mod net;
pub mod pool;
pub mod rng;
#[cfg(feature = "secret")]
pub mod secret;
#[cfg(feature = "smtp")]
pub mod smtp;
pub mod sync;
#[cfg(test)]
mod tests;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "web-socket")]
pub mod web_socket;
#[cfg(feature = "x509")]
pub mod x509;

pub use error::{Error, RecvError, SendError};
#[cfg(feature = "macros")]
pub use wtx_macros::*;

#[cfg(any(feature = "http2", feature = "tls"))]
const AFTER_CLOSE_TIMEOUT_MS: u64 = 100;
#[cfg(feature = "web-socket")]
const MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;

/// The wider length based on the selected host at compile time.
pub const SIMD_LEN: usize = _simd! {
  4 => { 4 },
  16 => { 16 },
  32 => { 32 },
  64 => { 64 }
};

/// Shortcut of [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;
