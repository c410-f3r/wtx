use crate::misc::Rng;

/// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
/// cryptographically secure.
pub trait CryptoRng: Rng {}
