//! Decode/Encode
//!
//! Groups different dependencies that transform different types of data.

#[macro_use]
mod macros;

mod csv;
mod de_controller;
mod dec_error;
mod decode;
mod encode;
pub mod format;
mod from_radix_10;
mod hex;
mod num_array;
mod percent_encoding;
pub mod protocol;

pub use csv::Csv;
pub use de_controller::DEController;
pub use dec_error::DecError;
pub use decode::{Decode, DecodeSeq};
pub use encode::Encode;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use hex::{HexDisplay, HexError, decode_hex_to_slice};
pub use num_array::{
  I16String, U8String, U16String, U32String, U64String, i16_string, u8_string, u16_string,
  u32_string, u64_string,
};
pub use percent_encoding::{AsciiSet, PercentDecode, PercentEncode};

/// Identifier used to track the number of issued requests.
pub type Id = usize;
