create_enum! {
  /// LZ77 sliding window size for the `permessage-deflate` extension from the IETF RFC 7692.
  #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
  #[repr(u8)]
  pub enum WindowBits {
    /// Eight
    Eight = 8,
    /// Nine
    Nine = 9,
    /// Ten
    Ten = 10,
    /// Eleven
    #[default]
    Eleven = 11,
    /// Twelve
    Twelve = 12,
    /// Thirteen
    Thirteen = 13,
    /// Fourteen
    Fourteen = 14,
    /// Fifteen
    Fifteen = 15,
  }
}

impl WindowBits {
  /// Instance that represents the minimum allowed value.
  pub const MIN: Self = Self::Eight;
  /// Instance that represents the maximum allowed value.
  pub const MAX: Self = Self::Fifteen;
}

impl From<WindowBits> for &'static str {
  #[inline]
  fn from(from: WindowBits) -> Self {
    match from {
      WindowBits::Eight => "8",
      WindowBits::Nine => "9",
      WindowBits::Ten => "10",
      WindowBits::Eleven => "11",
      WindowBits::Twelve => "12",
      WindowBits::Thirteen => "13",
      WindowBits::Fourteen => "14",
      WindowBits::Fifteen => "15",
    }
  }
}
