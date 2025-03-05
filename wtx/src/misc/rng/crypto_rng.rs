use crate::misc::Rng;

/// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
/// cryptographically secure.
#[cfg(feature = "rand-compat")]
pub trait CryptoRng: rand_core::CryptoRng + Rng {}

/// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
/// cryptographically secure.
#[cfg(not(feature = "rand-compat"))]
pub trait CryptoRng: Rng {}
