//! Decode/Encode
//!
//! Groups different dependencies that transform different types of data.

#[macro_use]
mod macros;

mod codec_controller;
mod codec_error;
mod codec_mode;
mod csv;
mod decode;
mod encode;
pub mod format;
mod from_radix_10;
mod generic_codec;
mod hex;
mod num_array;
pub mod protocol;
mod url_encoding;

pub use codec_controller::CodecController;
pub use codec_error::CodecError;
pub use codec_mode::CodecMode;
pub use csv::Csv;
pub use decode::{Decode, DecodeSeq};
pub use encode::Encode;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use generic_codec::{GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper};
pub use hex::{HexDisplay, HexEncMode, HexError, decode_hex, encode_hex};
pub use num_array::{
  I8String, I16String, I32String, I64String, U8String, U16String, U32String, U64String, i8_string,
  i16_string, i32_string, i64_string, u8_string, u16_string, u32_string, u64_string,
};
pub use url_encoding::*;

/// Identifier used to track the number of issued requests.
pub type Id = u64;
