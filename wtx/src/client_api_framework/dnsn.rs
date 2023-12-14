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
#[cfg(feature = "miniserde")]
mod miniserde;
#[cfg(feature = "protobuf")]
mod protobuf;
#[cfg(feature = "rkyv")]
mod rkyv;
#[cfg(feature = "serde_json")]
mod serde_json;
#[cfg(feature = "serde-xml-rs")]
mod serde_xml_rs;
#[cfg(feature = "serde_yaml")]
mod serde_yaml;
mod serialize;
#[cfg(feature = "simd-json")]
mod simd_json;

#[cfg(feature = "borsh")]
pub use self::borsh::*;
#[cfg(feature = "miniserde")]
pub use self::miniserde::*;
#[cfg(feature = "protobuf")]
pub use self::protobuf::*;
#[cfg(feature = "rkyv")]
pub use self::rkyv::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
#[cfg(feature = "serde-xml-rs")]
pub use self::serde_xml_rs::*;
#[cfg(feature = "serde_yaml")]
pub use self::serde_yaml::*;
#[cfg(feature = "simd-json")]
pub use self::simd_json::*;
pub use deserialize::*;
pub use serialize::*;
