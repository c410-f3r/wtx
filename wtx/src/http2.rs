//! HTTP/2

// Many elements where heavily influenced by the code of the h2 repository
// (https://github.com/hyperium/h2) so thanks to the authors.

mod hpack_encoder;
mod huffman;
mod huffman_tables;
mod raw_header;

pub use huffman::{huffman_decode, huffman_encode};
pub(crate) use raw_header::RawHeader;
