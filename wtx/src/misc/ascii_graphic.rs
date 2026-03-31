/// A byte that is guarantee to be a printable ASCII character.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AsciiGraphic(u8);

impl AsciiGraphic {
  /// Space
  pub const SPACE: Self = Self(b' ');
  /// Zero
  pub const ZERO: Self = Self(b'0');

  /// Checks if `byte` is a graphic ASCII character.
  #[inline]
  pub const fn new(byte: u8) -> crate::Result<Self> {
    if !byte.is_ascii_graphic() {
      return Err(crate::Error::NonGraphicByte);
    }
    Ok(Self(byte))
  }
}

impl From<AsciiGraphic> for char {
  #[inline]
  fn from(value: AsciiGraphic) -> Self {
    char::from(value.0)
  }
}

impl From<AsciiGraphic> for u8 {
  #[inline]
  fn from(value: AsciiGraphic) -> Self {
    value.0
  }
}

impl TryFrom<&[u8]> for AsciiGraphic {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let [byte] = value else {
      return Err(crate::Error::NonGraphicByte);
    };
    Self::new(*byte)
  }
}

impl TryFrom<&str> for AsciiGraphic {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    value.as_bytes().try_into()
  }
}

impl TryFrom<char> for AsciiGraphic {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: char) -> Result<Self, Self::Error> {
    Self::new(value.try_into().map_err(|_err| crate::Error::NonGraphicByte)?)
  }
}

impl TryFrom<u8> for AsciiGraphic {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}
