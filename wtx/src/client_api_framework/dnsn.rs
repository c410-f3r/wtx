//! # `D`eserializatio`N`/`S`erializatio`N`
//!
//! Abstracts different serialization/deserialization frameworks to enhance de-coupling,
//! enable choice and improve experimentation.

#[cfg(test)]
#[macro_use]
mod tests;

#[cfg(feature = "borsh")]
mod borsh;
mod deserialize;
#[cfg(feature = "protobuf")]
mod protobuf;
#[cfg(feature = "rkyv")]
mod rkyv;
#[cfg(feature = "serde_json")]
mod serde_json;
mod serialize;
#[cfg(feature = "simd-json")]
mod simd_json;

#[cfg(feature = "borsh")]
pub use self::borsh::*;
#[cfg(feature = "protobuf")]
pub use self::protobuf::*;
#[cfg(feature = "rkyv")]
pub use self::rkyv::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
#[cfg(feature = "simd-json")]
pub use self::simd_json::*;
pub use deserialize::*;
pub use serialize::*;
