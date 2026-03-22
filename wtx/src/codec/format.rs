//! Abstracts different serialization/deserialization frameworks to enhance de-coupling.

#[cfg(all(feature = "client-api-framework", test))]
#[macro_use]
mod tests;

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf;
#[cfg(feature = "serde_json")]
mod serde_json;

#[cfg(feature = "quick-protobuf")]
pub use self::quick_protobuf::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
