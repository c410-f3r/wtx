use rand_core::RngCore;

macro_rules! implement {
  ($struct:ty) => {
    impl crate::misc::rng::Rng for $struct {
      #[inline]
      fn u8(&mut self) -> u8 {
        let [a, ..] = self.next_u32().to_be_bytes();
        a
      }

      #[inline]
      fn u8_4(&mut self) -> [u8; 4] {
        self.next_u32().to_be_bytes()
      }

      #[inline]
      fn u8_8(&mut self) -> [u8; 8] {
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h]
      }

      #[inline]
      fn u8_16(&mut self) -> [u8; 16] {
        let [a, b, c, d, e, f, g, h] = self.next_u64().to_be_bytes();
        let [i, j, k, l, m, n, o, p] = self.next_u64().to_be_bytes();
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
      }
    }
  };
}

implement!(rand_chacha::ChaCha8Rng);
implement!(rand_chacha::ChaCha12Rng);
implement!(rand_chacha::ChaCha20Rng);
