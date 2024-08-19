_create_enum! {
  /// LZ77 sliding window size for the `permessage-deflate` extension from the IETF RFC 7692.
  #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
  pub enum WindowBits<u8> {
    /// Eight
    Eight = (8),
    /// Nine
    Nine = (9),
    /// Ten
    Ten = (10),
    /// Eleven
    #[default]
    Eleven = (11),
    /// Twelve
    Twelve = (12),
    /// Thirteen
    Thirteen = (13),
    /// Fourteen
    Fourteen = (14),
    /// Fifteen
    Fifteen = (15),
  }
}

impl WindowBits {
  /// Instance that represents the minimum allowed value.
  pub const MIN: Self = Self::Eight;
  /// Instance that represents the maximum allowed value.
  pub const MAX: Self = Self::Fifteen;
}
