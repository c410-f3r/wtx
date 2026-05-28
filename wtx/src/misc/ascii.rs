use core::fmt::{Display, Formatter, Write as _};

/// For example, `A` or `b`.
pub type AsciiAlphabetic = Ascii<0>;
/// For example, `A`, `b`, or `1`.
pub type AsciiAlphaNumeric = Ascii<1>;
/// For example, `\n` or `\t`.
pub type AsciiControl = Ascii<2>;
/// For example, `0` or `9`.
pub type AsciiDigit = Ascii<3>;
/// Arbitrary ASCII.
pub type AsciiGeneric = Ascii<4>;
/// For example, `;`, `1` or `!`.
pub type AsciiGraphic = Ascii<5>;
/// For example, `0`, `9`, `A`, or `f`.
pub type AsciiHexDigit = Ascii<6>;
/// For example, `a` or `z`.
pub type AsciiLowercase = Ascii<7>;
/// For example, `!` or `;`.
pub type AsciiPunctuation = Ascii<8>;
/// For example, `A` or `Z`.
pub type AsciiUppercase = Ascii<9>;
/// For example, ` ` or `\n`.
pub type AsciiWhitespace = Ascii<10>;

/// ASCII error
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AsciiError {
  /// Not an ASCII alphabetic character.
  NotAlphabetic,
  /// Not an ASCII alphanumeric character.
  NotAlphaNumeric,
  /// Not valid ASCII.
  NotAscii,
  /// Not an ASCII control character.
  NotControl,
  /// Not an ASCII digit.
  NotDigit,
  /// Not an ASCII graphic character.
  NotGraphic,
  /// Not an ASCII hex digit.
  NotHexDigit,
  /// Not an ASCII lowercase character.
  NotLowercase,
  /// Not an ASCII punctuation character.
  NotPunctuation,
  /// Character should be composed by a single byte when converting.
  NonUnitChar,
  /// Slice should be composed by a single byte when converting.
  NonUnitSlice,
  /// Not an ASCII uppercase character.
  NotUppercase,
  /// Not an ASCII whitespace character.
  NotWhitespace,
}

/// ASCII type
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AsciiTy {
  /// Alphabetic
  Alphabetic,
  /// `AlphaNumeric`
  AlphaNumeric,
  /// Control
  Control,
  /// Digit
  Digit,
  /// Generic
  Generic,
  /// Graphic
  Graphic,
  /// `HexDigit`
  HexDigit,
  /// Lowercase
  Lowercase,
  /// Punctuation
  Punctuation,
  /// Uppercase
  Uppercase,
  /// Whitespace
  Whitespace,
}

/// A byte that is guaranteed to match a specific ASCII category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ascii<const AT: u8>(u8);

impl<const AT: u8> Ascii<AT> {
  /// Generic constructor
  #[inline]
  pub const fn new(byte: u8) -> Result<Self, AsciiError> {
    match AsciiTy::from_u8(AT) {
      AsciiTy::Alphabetic => {
        if byte.is_ascii_alphabetic() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotAlphabetic)
        }
      }
      AsciiTy::AlphaNumeric => {
        if byte.is_ascii_alphanumeric() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotAlphaNumeric)
        }
      }
      AsciiTy::Control => {
        if byte.is_ascii_control() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotControl)
        }
      }
      AsciiTy::Digit => {
        if byte.is_ascii_digit() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotDigit)
        }
      }
      AsciiTy::Generic => {
        if byte.is_ascii() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotAscii)
        }
      }
      AsciiTy::Graphic => {
        if byte.is_ascii_graphic() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotGraphic)
        }
      }
      AsciiTy::HexDigit => {
        if byte.is_ascii_hexdigit() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotHexDigit)
        }
      }
      AsciiTy::Lowercase => {
        if byte.is_ascii_lowercase() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotLowercase)
        }
      }
      AsciiTy::Punctuation => {
        if byte.is_ascii_punctuation() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotPunctuation)
        }
      }
      AsciiTy::Uppercase => {
        if byte.is_ascii_uppercase() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotUppercase)
        }
      }
      AsciiTy::Whitespace => {
        if byte.is_ascii_whitespace() {
          Ok(Self(byte))
        } else {
          Err(AsciiError::NotWhitespace)
        }
      }
    }
  }

  /// Returns the underlying byte.
  #[inline]
  pub const fn as_u8(self) -> u8 {
    self.0
  }

  /// Returns the underlying byte as a `char`.
  #[inline]
  pub fn as_char(self) -> char {
    char::from(self.0)
  }
}

impl AsciiAlphabetic {
  /// See [`AsciiAlphabetic`].
  #[inline]
  pub const fn alphabetic(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiAlphaNumeric {
  /// See [`AsciiAlphaNumeric`].
  #[inline]
  pub const fn alpha_numeric(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiControl {
  /// See [`AsciiControl`].
  #[inline]
  pub const fn control(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiDigit {
  /// See [`AsciiDigit`].
  #[inline]
  pub const fn digit(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiGeneric {
  /// See [`AsciiGeneric`].
  #[inline]
  pub const fn generic(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiGraphic {
  /// See [`AsciiGraphic`].
  #[inline]
  pub const fn graphic(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiHexDigit {
  /// See [`AsciiHexDigit`].
  #[inline]
  pub const fn hex_digit(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiLowercase {
  /// See [`AsciiLowercase`].
  #[inline]
  pub const fn lowercase(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiPunctuation {
  /// See [`AsciiPunctuation`].
  #[inline]
  pub const fn punctuation(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiUppercase {
  /// See [`AsciiUppercase`].
  #[inline]
  pub const fn uppercase(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl AsciiWhitespace {
  /// See [`AsciiWhitespace`].
  #[inline]
  pub const fn whitespace(byte: u8) -> Result<Self, AsciiError> {
    Self::new(byte)
  }
}

impl<const AT: u8> Display for Ascii<AT> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_char(self.as_char())
  }
}

impl<const AT: u8> From<Ascii<AT>> for char {
  #[inline]
  fn from(value: Ascii<AT>) -> Self {
    value.as_char()
  }
}

impl<const AT: u8> From<Ascii<AT>> for u8 {
  #[inline]
  fn from(value: Ascii<AT>) -> Self {
    value.0
  }
}

impl<const AT: u8> TryFrom<&[u8]> for Ascii<AT> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let [byte] = value else {
      return Err(AsciiError::NonUnitSlice.into());
    };
    Ok(Self::new(*byte)?)
  }
}

impl<const AT: u8> TryFrom<&str> for Ascii<AT> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    value.as_bytes().try_into()
  }
}

impl<const AT: u8> TryFrom<char> for Ascii<AT> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: char) -> Result<Self, Self::Error> {
    Ok(Self::new(value.try_into().map_err(|_err| AsciiError::NonUnitChar)?)?)
  }
}

impl<const AT: u8> TryFrom<u8> for Ascii<AT> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Ok(Self::new(value)?)
  }
}

impl AsciiTy {
  pub(crate) const fn from_u8(num: u8) -> Self {
    match num {
      0 => Self::Alphabetic,
      1 => Self::AlphaNumeric,
      2 => Self::Control,
      3 => Self::Digit,
      4 => Self::Generic,
      5 => Self::Graphic,
      6 => Self::HexDigit,
      7 => Self::Lowercase,
      8 => Self::Punctuation,
      9 => Self::Uppercase,
      _ => Self::Whitespace,
    }
  }
}

macro_rules! create_constants {
  (
    $($ty:ty),+ $(,)? {
      $($name:ident = $val:expr;)+
    }
  ) => {
    create_constants!(@expand { $($ty),+ } { $($name = $val;)+ });
  };
  (@expand { $($ty:ty),+ } $consts:tt) => {
    $(create_constants!(@impl $ty, $consts);)+
  };
  (@impl $ty:ty, { $($name:ident = $val:expr;)+ }) => {
    impl $ty {
      $(
        #[doc = stringify!($name)]
        pub const $name: Self = Self($val);
      )+
    }
  };
}

macro_rules! create_conversions {
  (
    $($ty:ident),* $(,)?
  ) => {
    $(
      impl From<$ty> for AsciiGeneric {
        #[inline]
        fn from(value: $ty) -> Self {
          Self(value.as_u8())
        }
      }

      impl TryFrom<AsciiGeneric> for $ty {
        type Error = crate::Error;

        #[inline]
        fn try_from(value: AsciiGeneric) -> Result<Self, Self::Error> {
          Ok(Self::new(value.as_u8())?)
        }
      }
    )*
  };
}

macro_rules! impl_default {
  (
    $(($type:ident, $char:expr)),* $(,)?
  ) => {
    $(
      impl Default for $type {
        #[inline]
        fn default() -> Self {
          Self($char)
        }
      }
    )*
  };
}

create_constants! {
  AsciiGeneric {
    NULL = 0;
    START_OF_HEADING = 1;
    START_OF_TEXT = 2;
    END_OF_TEXT = 3;
    END_OF_TRANSMISSION = 4;
    ENQUIRY = 5;
    ACKNOWLEDGE = 6;
    BELL = 7;
    BACKSPACE = 8;
    TAB = b'\t';
    LINE_FEED = b'\n';
    VERTICAL_TAB = 11;
    FORM_FEED = 12;
    CARRIAGE_RETURN = b'\r';
    SHIFT_OUT = 14;
    SHIFT_IN = 15;
    DATA_LINK_ESCAPE = 16;
    DEVICE_CONTROL_1 = 17;
    DEVICE_CONTROL_2 = 18;
    DEVICE_CONTROL_3 = 19;
    DEVICE_CONTROL_4 = 20;
    NEGATIVE_ACKNOWLEDGE = 21;
    SYNCHRONOUS_IDLE = 22;
    END_OF_TRANSMISSION_BLOCK = 23;
    CANCEL = 24;
    END_OF_MEDIUM = 25;
    SUBSTITUTE = 26;
    ESCAPE = 27;
    FILE_SEPARATOR = 28;
    GROUP_SEPARATOR = 29;
    RECORD_SEPARATOR = 30;
    UNIT_SEPARATOR = 31;
    SPACE = b' ';
    EXCLAMATION_MARK = b'!';
    DOUBLE_QUOTE = b'"';
    HASH = b'#';
    DOLLAR = b'$';
    PERCENT = b'%';
    AMPERSAND = b'&';
    SINGLE_QUOTE = b'\'';
    LEFT_PARENTHESIS = b'(';
    RIGHT_PARENTHESIS = b')';
    ASTERISK = b'*';
    PLUS = b'+';
    COMMA = b',';
    MINUS = b'-';
    DOT = b'.';
    SLASH = b'/';
    ZERO = b'0';
    ONE = b'1';
    TWO = b'2';
    THREE = b'3';
    FOUR = b'4';
    FIVE = b'5';
    SIX = b'6';
    SEVEN = b'7';
    EIGHT = b'8';
    NINE = b'9';
    COLON = b':';
    SEMICOLON = b';';
    LESS_THAN = b'<';
    EQUAL = b'=';
    GREATER_THAN = b'>';
    QUESTION_MARK = b'?';
    AT = b'@';
    A_UPPER = b'A';
    B_UPPER = b'B';
    C_UPPER = b'C';
    D_UPPER = b'D';
    E_UPPER = b'E';
    F_UPPER = b'F';
    G_UPPER = b'G';
    H_UPPER = b'H';
    I_UPPER = b'I';
    J_UPPER = b'J';
    K_UPPER = b'K';
    L_UPPER = b'L';
    M_UPPER = b'M';
    N_UPPER = b'N';
    O_UPPER = b'O';
    P_UPPER = b'P';
    Q_UPPER = b'Q';
    R_UPPER = b'R';
    S_UPPER = b'S';
    T_UPPER = b'T';
    U_UPPER = b'U';
    V_UPPER = b'V';
    W_UPPER = b'W';
    X_UPPER = b'X';
    Y_UPPER = b'Y';
    Z_UPPER = b'Z';
    LEFT_BRACKET = b'[';
    BACKSLASH = b'\\';
    RIGHT_BRACKET = b']';
    CARET = b'^';
    UNDERSCORE = b'_';
    BACKTICK = b'`';
    A_LOWER = b'a';
    B_LOWER = b'b';
    C_LOWER = b'c';
    D_LOWER = b'd';
    E_LOWER = b'e';
    F_LOWER = b'f';
    G_LOWER = b'g';
    H_LOWER = b'h';
    I_LOWER = b'i';
    J_LOWER = b'j';
    K_LOWER = b'k';
    L_LOWER = b'l';
    M_LOWER = b'm';
    N_LOWER = b'n';
    O_LOWER = b'o';
    P_LOWER = b'p';
    Q_LOWER = b'q';
    R_LOWER = b'r';
    S_LOWER = b's';
    T_LOWER = b't';
    U_LOWER = b'u';
    V_LOWER = b'v';
    W_LOWER = b'w';
    X_LOWER = b'x';
    Y_LOWER = b'y';
    Z_LOWER = b'z';
    LEFT_BRACE = b'{';
    PIPE = b'|';
    RIGHT_BRACE = b'}';
    TILDE = b'~';
    DELETE = 127;
  }
}

create_conversions! {
  AsciiAlphabetic,
  AsciiAlphaNumeric,
  AsciiControl,
  AsciiDigit,
  AsciiGraphic,
  AsciiHexDigit,
  AsciiLowercase,
  AsciiPunctuation,
  AsciiUppercase,
  AsciiWhitespace,
}

impl_default! {
  (AsciiAlphabetic, b'a'),
  (AsciiAlphaNumeric, b'0'),
  (AsciiControl, 0),
  (AsciiDigit, b'0'),
  (AsciiGeneric, b'A'),
  (AsciiGraphic, b'!'),
  (AsciiHexDigit, b'0'),
  (AsciiLowercase, b'a'),
  (AsciiPunctuation, b'.'),
  (AsciiUppercase, b'A'),
  (AsciiWhitespace, b' ')
}
