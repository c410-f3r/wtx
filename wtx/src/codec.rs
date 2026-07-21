//! Decode/Encode
//!
//! Groups different dependencies that transform different types of data.

#[macro_use]
mod macros;

pub(crate) mod alphabet;
mod base64;
mod codec_controller;
mod codec_error;
mod codec_mode;
mod compression;
mod compression_level;
mod csv;
mod decode;
mod decompression;
mod encode;
pub mod format;
mod from_radix_10;
mod generic_codec;
mod hex;
mod misc;
mod num_array;
mod pem;
pub mod protocol;
mod url_encoding;

pub use base64::{
  Base64Alphabet, Base64Error, base64_decode, base64_decoded_len_ub, base64_encode,
  base64_encoded_len,
};
pub use codec_controller::CodecController;
pub use codec_error::CodecError;
pub use codec_mode::CodecMode;
pub use compression::{Compression, CompressionFlush};
pub use compression_level::CompressionLevel;
pub use csv::Csv;
pub use decode::{Decode, DecodeSeq};
pub use decompression::{Decompression, DecompressionFlush};
pub use encode::Encode;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use generic_codec::{DecodeWrapper, EncodeWrapper, GenericCodec};
pub use hex::*;
pub use misc::{
  decode_base64_into_buffer, decode_hex_into_buffer, encode_base64_into_buffer,
  encode_hex_into_buffer,
};
pub use num_array::*;
pub use pem::Pem;
pub use url_encoding::*;

/// Identifier used to track the number of issued requests.
pub type Id = u64;
