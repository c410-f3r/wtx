#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

#[macro_use]
mod macros;

mod array_chunks;
mod async_bounds;
mod buffer;
mod cache;
mod error;
mod expected_header;
pub mod http;
mod misc;
mod partitioned_buffer;
pub mod rng;
#[cfg(feature = "tracing")]
mod role;
mod stream;
pub mod web_socket;

pub use array_chunks::ArrayChunksMut;
pub use async_bounds::AsyncBounds;
pub use cache::Cache;
pub use error::Error;
pub use expected_header::ExpectedHeader;
pub use misc::uri_parts::UriParts;
pub use partitioned_buffer::PartitionedBuffer;
pub use stream::{BytesStream, Stream};

pub(crate) const DFLT_PARTITIONED_BUFFER_LEN: usize = 32 * 1024;
pub(crate) const MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;

/// Shortcut of [core::result::Result<T, Error>].
pub type Result<T> = core::result::Result<T, Error>;
