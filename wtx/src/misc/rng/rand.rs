use rand::Rng;

macro_rules! implement {
  ($struct:ty) => {
    impl crate::misc::rng::Rng for $struct {
      #[inline]
      fn u8(&mut self) -> u8 {
        self.r#gen()
      }

      #[inline]
      fn u8_4(&mut self) -> [u8; 4] {
        self.r#gen()
      }

      #[inline]
      fn u8_8(&mut self) -> [u8; 8] {
        self.r#gen()
      }

      #[inline]
      fn u8_16(&mut self) -> [u8; 16] {
        self.r#gen()
      }
    }
  };
}

implement!(rand::rngs::mock::StepRng);
implement!(rand::rngs::SmallRng);
