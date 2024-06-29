//! A flexible client API framework for writing asynchronous, fast, organizable, scalable and maintainable applications.

#[macro_use]
mod macros;

mod api;
mod client_api_framework_error;
pub mod data_format;
pub mod dnsn;
pub mod misc;
pub mod network;
pub mod pkg;
mod tests;

pub use api::Api;
pub use client_api_framework_error::ClientApiFrameworkError;

/// Identifier used to track the number of issued requests.
pub type Id = usize;
