#![cfg_attr(feature = "_bench", allow(soft_unstable))]
#![cfg_attr(feature = "_bench", feature(test))]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;
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
#[cfg(feature = "http1")]
mod http1;
pub mod misc;
#[cfg(feature = "pool-manager")]
pub mod pool_manager;
pub mod rng;
#[cfg(feature = "web-socket")]
pub mod web_socket;

pub use error::Error;

pub(crate) const DFLT_PARTITIONED_BUFFER_LEN: usize = 32 * 1024;
pub(crate) const _MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;

/// Shortcut of [core::result::Result<T, Error>].
pub type Result<T> = core::result::Result<T, Error>;
