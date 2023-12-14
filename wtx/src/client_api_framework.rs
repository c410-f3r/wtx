//! A flexible client API framework for writing asynchronous, fast, organizable, scalable and maintainable applications.

#[macro_use]
mod macros;

mod api;
pub mod data_format;
pub mod dnsn;
pub mod misc;
pub mod network;
pub mod pkg;

pub use api::Api;

/// Identifier used to track the number of issued requests.
pub type Id = usize;
