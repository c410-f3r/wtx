macro_rules! _implement_rand {
  ($struct:ty, $seed:ty) => {
    impl crate::rng::CryptoRng for $struct {}

    impl crate::rng::Rng for $struct {
      #[inline]
      fn u8_4(&mut self) -> [u8; 4] {
        use chacha20::rand_core::Rng;
        self.next_u32().to_be_bytes()
      }

      #[inline]
      fn u8_8(&mut self) -> [u8; 8] {
        use chacha20::rand_core::Rng;
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h]
      }

      #[inline]
      fn u8_16(&mut self) -> [u8; 16] {
        use chacha20::rand_core::Rng;
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        let [i, j, k, l, m, n, o, p] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
      }

      #[inline]
      fn u8_32(&mut self) -> [u8; 32] {
        use chacha20::rand_core::Rng;
        let [b0, b1, b2, b3, b4, b5, b6, b7] = self.next_u64().to_be_bytes();
        let [b8, b9, b10, b11, b12, b13, b14, b15] = self.next_u64().to_be_bytes();
        let [b16, b17, b18, b19, b20, b21, b22, b23] = self.next_u64().to_be_bytes();
        let [b24, b25, b26, b27, b28, b29, b30, b31] = self.next_u64().to_be_bytes();
        [
          b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19,
          b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31,
        ]
      }
    }

    impl crate::rng::SeedableRng for $struct {
      type Seed = $seed;

      #[inline]
      fn from_seed(seed: Self::Seed) -> crate::Result<Self> {
        Ok(<Self as chacha20::rand_core::SeedableRng>::from_seed(seed))
      }
    }
  };
}

_implement_rand!(chacha20::ChaCha20Rng, [u8; 32]);
_implement_rand!(chacha20::ChaCha12Rng, [u8; 32]);
_implement_rand!(chacha20::ChaCha8Rng, [u8; 32]);
