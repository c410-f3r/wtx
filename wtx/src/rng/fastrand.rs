impl crate::rng::Rng for fastrand::Rng {
  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    self.u32(0..=u32::MAX).to_be_bytes()
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    self.u64(0..=u64::MAX).to_be_bytes()
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    self.u128(0..=u128::MAX).to_be_bytes()
  }

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] =
      self.u128(0..=u128::MAX).to_be_bytes();
    let [b16, b17, b18, b19, b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31] =
      self.u128(0..=u128::MAX).to_be_bytes();
    [
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19,
      b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31,
    ]
  }
}
