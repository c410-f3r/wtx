#![allow(incomplete_features, reason = "essential async feature")]
#![cfg_attr(feature = "_bench", allow(soft_unstable))]
#![cfg_attr(feature = "_bench", feature(test))]
#![doc = include_str!("../README.md")]
#![feature(macro_metavar_expr, noop_waker, return_type_notation)]
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;
#[cfg(all(feature = "_bench", test))]
extern crate test;

#[macro_use]
mod macros;

#[cfg(all(feature = "_bench", test))]
pub(crate) mod bench;
#[cfg(feature = "client-api-framework")]
pub mod client_api_framework;
#[cfg(feature = "database")]
pub mod database;
mod error;
pub mod http;
#[cfg(feature = "http2")]
pub mod http2;
pub mod misc;
#[cfg(feature = "pool")]
pub mod pool;
pub mod rng;
mod tuple_impls;
#[cfg(feature = "web-socket")]
pub mod web_socket;

pub use error::Error;
#[cfg(feature = "std")]
pub use error::VarError;

pub(crate) const _MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;

/// Shortcut of [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;
