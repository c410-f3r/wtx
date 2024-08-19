//! Groups different dependencies that transform different types of data.

#[macro_use]
mod macros;

mod data_transformation_error;
pub mod dnsn;
pub mod format;
mod seq_visitor;

pub use data_transformation_error::DataTransformationError;

/// Identifier used to track the number of issued requests.
pub type Id = usize;
