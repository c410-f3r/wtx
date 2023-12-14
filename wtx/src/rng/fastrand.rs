use core::ops::RangeInclusive;

const RANGE: RangeInclusive<u8> = 0..=255;

impl crate::rng::Rng for fastrand::Rng {
  #[inline]
  fn u8(&mut self) -> u8 {
    self.u8(RANGE)
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    [self.u8(RANGE), self.u8(RANGE), self.u8(RANGE), self.u8(RANGE)]
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    [
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
      self.u8(RANGE),
    ]
  }
}
