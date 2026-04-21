use crate::rng::Rng;

cfg_select! {
  feature = "rand_core" => {
    /// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
    /// cryptographically secure.
    pub trait CryptoRng: rand_core::CryptoRng + Rng {}

    impl<T> CryptoRng for &mut T where T: rand_core::CryptoRng + Rng {}
  }
  _ => {
    /// Marker trait used to indicate that an [`Rng`] implementation is supposed to be
    /// cryptographically secure.
    pub trait CryptoRng: Rng {}

    impl<T> CryptoRng for &mut T where T: Rng {}
  }
}
