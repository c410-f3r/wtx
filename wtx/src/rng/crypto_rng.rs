use crate::rng::Rng;

/// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
/// cryptographically secure.
#[cfg(feature = "rand-compat")]
pub trait CryptoRng: rand_core::CryptoRng + Rng {}

/// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
/// cryptographically secure.
#[cfg(not(feature = "rand-compat"))]
pub trait CryptoRng: Rng {}

#[cfg(feature = "rand-compat")]
impl<T> CryptoRng for &mut T where T: rand_core::CryptoRng + Rng {}
#[cfg(not(feature = "rand-compat"))]
impl<T> CryptoRng for &mut T where T: Rng {}
