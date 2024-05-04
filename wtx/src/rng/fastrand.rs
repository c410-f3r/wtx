impl crate::rng::Rng for fastrand::Rng {
  #[inline]
  fn u8(&mut self) -> u8 {
    self.u8(0..=u8::MAX)
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    self.u32(0..=u32::MAX).to_be_bytes()
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    self.u128(0..=u128::MAX).to_be_bytes()
  }
}
