//! Abstracts different serialization/deserialization frameworks to enhance de-coupling.

#[cfg(all(feature = "client-api-framework", test))]
#[macro_use]
mod tests;

#[cfg(feature = "borsh")]
mod borsh;
mod de;
mod decode_wrapper;
mod encode_wrapper;
mod hex;
#[cfg(feature = "quick-protobuf")]
mod quick_protobuf;
#[cfg(feature = "serde_json")]
mod serde_json;

#[cfg(feature = "borsh")]
pub use self::borsh::*;
#[cfg(feature = "quick-protobuf")]
pub use self::quick_protobuf::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
pub use de::De;
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use hex::Hex;
