//! HTTP/2

// Many elements where influenced by the code of the h2 repository (https://github.com/hyperium/h2)
// so thanks to the authors.

mod hpack_decoding_headers;
mod hpack_encoding_headers;
mod hpack_header;
mod hpack_static_headers;
mod huffman;
mod huffman_tables;

pub use hpack_decoding_headers::HpackDecodingHeaders;
pub use hpack_encoding_headers::HpackEncodingHeaders;
pub use hpack_header::{HpackHeaderBasic, HpackHeaderName};
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub(crate) use huffman::{huffman_decode, huffman_encode};
