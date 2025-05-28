macro_rules! _implement_rand {
  ($struct:ty, $seed:ty) => {
    impl crate::rng::CryptoRng for $struct {}

    impl crate::rng::Rng for $struct {
      #[inline]
      fn u8(&mut self) -> u8 {
        use rand_chacha::rand_core::RngCore;
        let [a, ..] = self.next_u32().to_be_bytes();
        a
      }

      #[inline]
      fn u8_4(&mut self) -> [u8; 4] {
        use rand_chacha::rand_core::RngCore;
        self.next_u32().to_be_bytes()
      }

      #[inline]
      fn u8_8(&mut self) -> [u8; 8] {
        use rand_chacha::rand_core::RngCore;
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h]
      }

      #[inline]
      fn u8_16(&mut self) -> [u8; 16] {
        use rand_chacha::rand_core::RngCore;
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        let [i, j, k, l, m, n, o, p] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
      }
    }

    impl crate::rng::SeedableRng for $struct {
      type Seed = $seed;

      #[inline]
      fn from_seed(seed: Self::Seed) -> crate::Result<Self> {
        Ok(<Self as rand_chacha::rand_core::SeedableRng>::from_seed(seed))
      }
    }
  };
}

_implement_rand!(rand_chacha::ChaCha8Rng, [u8; 32]);
_implement_rand!(rand_chacha::ChaCha12Rng, [u8; 32]);
_implement_rand!(rand_chacha::ChaCha20Rng, [u8; 32]);
