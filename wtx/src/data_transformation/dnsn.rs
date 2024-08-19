//! # `D`eserializatio`N`/`S`erializatio`N`
//!
//! Abstracts different serialization/deserialization frameworks to enhance de-coupling,
//! enable choice and improve experimentation.

#[cfg(all(feature = "client-api-framework", test))]
#[macro_use]
mod tests;

#[cfg(feature = "borsh")]
mod borsh;
mod deserialize;
#[cfg(feature = "quick-protobuf")]
mod quick_protobuf;
#[cfg(feature = "rkyv")]
mod rkyv;
#[cfg(feature = "serde_json")]
mod serde_json;
mod serialize;

#[cfg(feature = "borsh")]
pub use self::borsh::*;
#[cfg(feature = "quick-protobuf")]
pub use self::quick_protobuf::*;
#[cfg(feature = "rkyv")]
pub use self::rkyv::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
pub use deserialize::*;
pub use serialize::*;
