/// Arbitrary ASCII.
pub type Ascii = AsciiGeneric<false>;
/// For example, `A`, `1` or `!`.
pub type AsciiGraphic = AsciiGeneric<true>;

/// A byte that is guarantee to be an ASCII character.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AsciiGeneric<const IS_GRAPHIC: bool>(u8);

impl<const IS_GRAPHIC: bool> AsciiGeneric<IS_GRAPHIC> {
  /// Ampersand
  pub const AMPERSAND: Self = Self(b'&');
  /// At
  pub const AT: Self = Self(b'@');
  /// Colon
  pub const COLON: Self = Self(b':');
  /// Comma
  pub const COMMA: Self = Self(b',');
  /// Double quote
  pub const DOUBLE_QUOTE: Self = Self(b'"');
  /// Space
  pub const EQUAL: Self = Self(b'=');
  /// Space
  pub const NUMBER_SIGN: Self = Self(b'#');
  /// Opening brace
  pub const OPENING_BRACE: Self = Self(b'{');
  /// Semicolon
  pub const SEMICOLON: Self = Self(b';');
  /// Slash
  pub const SLASH: Self = Self(b'/');
  /// Space
  pub const SPACE: Self = Self(b' ');
  /// Zero
  pub const ZERO: Self = Self(b'0');

  /// Checks if `byte` is a graphic ASCII character.
  #[inline]
  pub const fn new(byte: u8) -> crate::Result<Self> {
    if IS_GRAPHIC {
      if !byte.is_ascii_graphic() {
        return Err(crate::Error::NonAsciiByte);
      }
    } else if !byte.is_ascii() {
      return Err(crate::Error::NonAsciiByte);
    }
    Ok(Self(byte))
  }
}

impl Ascii {
  /// Null
  pub const NULL: Self = Self(0);
}

impl<const IS_GRAPHIC: bool> From<AsciiGeneric<IS_GRAPHIC>> for char {
  #[inline]
  fn from(value: AsciiGeneric<IS_GRAPHIC>) -> Self {
    char::from(value.0)
  }
}

impl<const IS_GRAPHIC: bool> From<AsciiGeneric<IS_GRAPHIC>> for u8 {
  #[inline]
  fn from(value: AsciiGeneric<IS_GRAPHIC>) -> Self {
    value.0
  }
}

impl<const IS_GRAPHIC: bool> TryFrom<&[u8]> for AsciiGeneric<IS_GRAPHIC> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let [byte] = value else {
      return Err(crate::Error::NonAsciiByte);
    };
    Self::new(*byte)
  }
}

impl<const IS_GRAPHIC: bool> TryFrom<&str> for AsciiGeneric<IS_GRAPHIC> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    value.as_bytes().try_into()
  }
}

impl<const IS_GRAPHIC: bool> TryFrom<char> for AsciiGeneric<IS_GRAPHIC> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: char) -> Result<Self, Self::Error> {
    Self::new(value.try_into().map_err(|_err| crate::Error::NonAsciiByte)?)
  }
}

impl<const IS_GRAPHIC: bool> TryFrom<u8> for AsciiGeneric<IS_GRAPHIC> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}
