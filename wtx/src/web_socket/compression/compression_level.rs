create_enum! {
  /// A scale from 0 to 9 where 0 means "no compression" and 9 means "take as long as you'd like".
  #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
  #[repr(u8)]
  pub enum CompressionLevel {
    /// Zero
    Zero = 0,
    /// One
    One = 1,
    /// Two
    Two = 2,
    /// Three
    Three = 3,
    /// Four
    Four = 4,
    /// Five
    #[default]
    Five = 5,
    /// Six
    Six = 6,
    /// Seven
    Seven = 7,
    /// Eight
    Eight = 8,
    /// Nine
    Nine = 9,
  }
}

impl CompressionLevel {
  /// Instance that represents the minimum allowed value.
  pub const MIN: Self = Self::Zero;
  /// Instance that represents the maximum allowed value.
  pub const MAX: Self = Self::Nine;
}

#[cfg(feature = "flate2")]
impl From<CompressionLevel> for flate2::Compression {
  #[inline]
  fn from(from: CompressionLevel) -> Self {
    flate2::Compression::new(u8::from(from).into())
  }
}
