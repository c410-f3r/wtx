use crate::misc::const_ok;
use core::{
  fmt::{Display, Formatter, Write as _},
  marker::PhantomData,
  str::FromStr,
};

/// For example, `A` or `b`.
pub type AsciiAlphabetic = Ascii<AsciiAlphabeticParams>;
/// For example, `A`, `b`, or `1`.
pub type AsciiAlphaNumeric = Ascii<AsciiAlphaNumericParams>;
/// For example, `\n` or `\t`.
pub type AsciiControl = Ascii<AsciiControlParams>;
/// For example, `0` or `9`.
pub type AsciiDigit = Ascii<AsciiDigitParams>;
/// For example, `;`, `1` or `!`.
pub type AsciiGraphic = Ascii<AsciiGraphicParams>;
/// For example, `0`, `9`, `A`, or `f`.
pub type AsciiHexDigit = Ascii<AsciiHexDigitParams>;
/// For example, `a` or `z`.
pub type AsciiLowercase = Ascii<AsciiLowercaseParams>;
/// For example, `!` or `;`.
pub type AsciiPunctuation = Ascii<AsciiPunctuationParams>;
/// For example, `A` or `Z`.
pub type AsciiUppercase = Ascii<AsciiUppercaseParams>;
/// For example, ` ` or `\n`.
pub type AsciiWhitespace = Ascii<AsciiWhitespaceParams>;

/// ASCII error
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AsciiError {
  /// Not an ASCII alphabetic character.
  NotAlphabetic,
  /// Not an ASCII alphanumeric character.
  NotAlphaNumeric,
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
  /// For example, `A` or `b`.
  Alphabetic,
  /// For example, `A`, `b`, or `1`.
  AlphaNumeric,
  /// For example, `\n` or `\t`.
  Control,
  /// For example, `0` or `9`.
  Digit,
  /// For example, `;`, `1` or `!`.
  Graphic,
  /// For example, `0`, `9`, `A`, or `f`.
  HexDigit,
  /// For example, `a` or `z`.
  Lowercase,
  /// For example, `!` or `;`.
  Punctuation,
  /// For example, `A` or `Z`.
  Uppercase,
  /// For example, ` ` or `\n`.
  Whitespace,
}

/// A byte that is guaranteed to match a specific ASCII category.
#[derive(Debug, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Ascii<T>(u8, PhantomData<fn() -> T>);

impl<T> Ascii<T>
where
  T: AsciiParams,
{
  /// Null character
  pub const NULL: Self = const_ok(Self::new(0)).unwrap();
  /// Start of heading
  pub const START_OF_HEADING: Self = const_ok(Self::new(1)).unwrap();
  /// Start of text
  pub const START_OF_TEXT: Self = const_ok(Self::new(2)).unwrap();
  /// End of text
  pub const END_OF_TEXT: Self = const_ok(Self::new(3)).unwrap();
  /// End of transmission
  pub const END_OF_TRANSMISSION: Self = const_ok(Self::new(4)).unwrap();
  /// Enquiry
  pub const ENQUIRY: Self = const_ok(Self::new(5)).unwrap();
  /// Acknowledge
  pub const ACKNOWLEDGE: Self = const_ok(Self::new(6)).unwrap();
  /// Bell
  pub const BELL: Self = const_ok(Self::new(7)).unwrap();
  /// Backspace
  pub const BACKSPACE: Self = const_ok(Self::new(8)).unwrap();
  /// Horizontal tab
  pub const TAB: Self = const_ok(Self::new(b'\t')).unwrap();
  /// Line feed
  pub const LINE_FEED: Self = const_ok(Self::new(b'\n')).unwrap();
  /// Vertical tab
  pub const VERTICAL_TAB: Self = const_ok(Self::new(11)).unwrap();
  /// Form feed
  pub const FORM_FEED: Self = const_ok(Self::new(12)).unwrap();
  /// Carriage return
  pub const CARRIAGE_RETURN: Self = const_ok(Self::new(b'\r')).unwrap();
  /// Shift out
  pub const SHIFT_OUT: Self = const_ok(Self::new(14)).unwrap();
  /// Shift in
  pub const SHIFT_IN: Self = const_ok(Self::new(15)).unwrap();
  /// Data link escape
  pub const DATA_LINK_ESCAPE: Self = const_ok(Self::new(16)).unwrap();
  /// Device control 1
  pub const DEVICE_CONTROL_1: Self = const_ok(Self::new(17)).unwrap();
  /// Device control 2
  pub const DEVICE_CONTROL_2: Self = const_ok(Self::new(18)).unwrap();
  /// Device control 3
  pub const DEVICE_CONTROL_3: Self = const_ok(Self::new(19)).unwrap();
  /// Device control 4
  pub const DEVICE_CONTROL_4: Self = const_ok(Self::new(20)).unwrap();
  /// Negative acknowledge
  pub const NEGATIVE_ACKNOWLEDGE: Self = const_ok(Self::new(21)).unwrap();
  /// Synchronous idle
  pub const SYNCHRONOUS_IDLE: Self = const_ok(Self::new(22)).unwrap();
  /// End of transmission block
  pub const END_OF_TRANSMISSION_BLOCK: Self = const_ok(Self::new(23)).unwrap();
  /// Cancel
  pub const CANCEL: Self = const_ok(Self::new(24)).unwrap();
  /// End of medium
  pub const END_OF_MEDIUM: Self = const_ok(Self::new(25)).unwrap();
  /// Substitute
  pub const SUBSTITUTE: Self = const_ok(Self::new(26)).unwrap();
  /// Escape
  pub const ESCAPE: Self = const_ok(Self::new(27)).unwrap();
  /// File separator
  pub const FILE_SEPARATOR: Self = const_ok(Self::new(28)).unwrap();
  /// Group separator
  pub const GROUP_SEPARATOR: Self = const_ok(Self::new(29)).unwrap();
  /// Record separator
  pub const RECORD_SEPARATOR: Self = const_ok(Self::new(30)).unwrap();
  /// Unit separator
  pub const UNIT_SEPARATOR: Self = const_ok(Self::new(31)).unwrap();
  /// Space
  pub const SPACE: Self = const_ok(Self::new(b' ')).unwrap();
  /// Exclamation mark
  pub const EXCLAMATION_MARK: Self = const_ok(Self::new(b'!')).unwrap();
  /// Double quote
  pub const DOUBLE_QUOTE: Self = const_ok(Self::new(b'"')).unwrap();
  /// Hash
  pub const HASH: Self = const_ok(Self::new(b'#')).unwrap();
  /// Dollar sign
  pub const DOLLAR: Self = const_ok(Self::new(b'$')).unwrap();
  /// Percent sign
  pub const PERCENT: Self = const_ok(Self::new(b'%')).unwrap();
  /// Ampersand
  pub const AMPERSAND: Self = const_ok(Self::new(b'&')).unwrap();
  /// Single quote
  pub const SINGLE_QUOTE: Self = const_ok(Self::new(b'\'')).unwrap();
  /// Left parenthesis
  pub const LEFT_PARENTHESIS: Self = const_ok(Self::new(b'(')).unwrap();
  /// Right parenthesis
  pub const RIGHT_PARENTHESIS: Self = const_ok(Self::new(b')')).unwrap();
  /// Asterisk
  pub const ASTERISK: Self = const_ok(Self::new(b'*')).unwrap();
  /// Plus sign
  pub const PLUS: Self = const_ok(Self::new(b'+')).unwrap();
  /// Comma
  pub const COMMA: Self = const_ok(Self::new(b',')).unwrap();
  /// Minus sign
  pub const MINUS: Self = const_ok(Self::new(b'-')).unwrap();
  /// Period
  pub const DOT: Self = const_ok(Self::new(b'.')).unwrap();
  /// Forward slash
  pub const SLASH: Self = const_ok(Self::new(b'/')).unwrap();
  /// Digit 0
  pub const ZERO: Self = const_ok(Self::new(b'0')).unwrap();
  /// Digit 1
  pub const ONE: Self = const_ok(Self::new(b'1')).unwrap();
  /// Digit 2
  pub const TWO: Self = const_ok(Self::new(b'2')).unwrap();
  /// Digit 3
  pub const THREE: Self = const_ok(Self::new(b'3')).unwrap();
  /// Digit 4
  pub const FOUR: Self = const_ok(Self::new(b'4')).unwrap();
  /// Digit 5
  pub const FIVE: Self = const_ok(Self::new(b'5')).unwrap();
  /// Digit 6
  pub const SIX: Self = const_ok(Self::new(b'6')).unwrap();
  /// Digit 7
  pub const SEVEN: Self = const_ok(Self::new(b'7')).unwrap();
  /// Digit 8
  pub const EIGHT: Self = const_ok(Self::new(b'8')).unwrap();
  /// Digit 9
  pub const NINE: Self = const_ok(Self::new(b'9')).unwrap();
  /// Colon
  pub const COLON: Self = const_ok(Self::new(b':')).unwrap();
  /// Semicolon
  pub const SEMICOLON: Self = const_ok(Self::new(b';')).unwrap();
  /// Less than
  pub const LESS_THAN: Self = const_ok(Self::new(b'<')).unwrap();
  /// Equals sign
  pub const EQUAL: Self = const_ok(Self::new(b'=')).unwrap();
  /// Greater than
  pub const GREATER_THAN: Self = const_ok(Self::new(b'>')).unwrap();
  /// Question mark
  pub const QUESTION_MARK: Self = const_ok(Self::new(b'?')).unwrap();
  /// At symbol
  pub const AT: Self = const_ok(Self::new(b'@')).unwrap();
  /// Uppercase A
  pub const A_UPPER: Self = const_ok(Self::new(b'A')).unwrap();
  /// Uppercase B
  pub const B_UPPER: Self = const_ok(Self::new(b'B')).unwrap();
  /// Uppercase C
  pub const C_UPPER: Self = const_ok(Self::new(b'C')).unwrap();
  /// Uppercase D
  pub const D_UPPER: Self = const_ok(Self::new(b'D')).unwrap();
  /// Uppercase E
  pub const E_UPPER: Self = const_ok(Self::new(b'E')).unwrap();
  /// Uppercase F
  pub const F_UPPER: Self = const_ok(Self::new(b'F')).unwrap();
  /// Uppercase G
  pub const G_UPPER: Self = const_ok(Self::new(b'G')).unwrap();
  /// Uppercase H
  pub const H_UPPER: Self = const_ok(Self::new(b'H')).unwrap();
  /// Uppercase I
  pub const I_UPPER: Self = const_ok(Self::new(b'I')).unwrap();
  /// Uppercase J
  pub const J_UPPER: Self = const_ok(Self::new(b'J')).unwrap();
  /// Uppercase K
  pub const K_UPPER: Self = const_ok(Self::new(b'K')).unwrap();
  /// Uppercase L
  pub const L_UPPER: Self = const_ok(Self::new(b'L')).unwrap();
  /// Uppercase M
  pub const M_UPPER: Self = const_ok(Self::new(b'M')).unwrap();
  /// Uppercase N
  pub const N_UPPER: Self = const_ok(Self::new(b'N')).unwrap();
  /// Uppercase O
  pub const O_UPPER: Self = const_ok(Self::new(b'O')).unwrap();
  /// Uppercase P
  pub const P_UPPER: Self = const_ok(Self::new(b'P')).unwrap();
  /// Uppercase Q
  pub const Q_UPPER: Self = const_ok(Self::new(b'Q')).unwrap();
  /// Uppercase R
  pub const R_UPPER: Self = const_ok(Self::new(b'R')).unwrap();
  /// Uppercase S
  pub const S_UPPER: Self = const_ok(Self::new(b'S')).unwrap();
  /// Uppercase T
  pub const T_UPPER: Self = const_ok(Self::new(b'T')).unwrap();
  /// Uppercase U
  pub const U_UPPER: Self = const_ok(Self::new(b'U')).unwrap();
  /// Uppercase V
  pub const V_UPPER: Self = const_ok(Self::new(b'V')).unwrap();
  /// Uppercase W
  pub const W_UPPER: Self = const_ok(Self::new(b'W')).unwrap();
  /// Uppercase X
  pub const X_UPPER: Self = const_ok(Self::new(b'X')).unwrap();
  /// Uppercase Y
  pub const Y_UPPER: Self = const_ok(Self::new(b'Y')).unwrap();
  /// Uppercase Z
  pub const Z_UPPER: Self = const_ok(Self::new(b'Z')).unwrap();
  /// Left square bracket
  pub const LEFT_BRACKET: Self = const_ok(Self::new(b'[')).unwrap();
  /// Backslash
  pub const BACKSLASH: Self = const_ok(Self::new(b'\\')).unwrap();
  /// Right square bracket
  pub const RIGHT_BRACKET: Self = const_ok(Self::new(b']')).unwrap();
  /// Caret
  pub const CARET: Self = const_ok(Self::new(b'^')).unwrap();
  /// Underscore
  pub const UNDERSCORE: Self = const_ok(Self::new(b'_')).unwrap();
  /// Backtick
  pub const BACKTICK: Self = const_ok(Self::new(b'`')).unwrap();
  /// Lowercase a
  pub const A_LOWER: Self = const_ok(Self::new(b'a')).unwrap();
  /// Lowercase b
  pub const B_LOWER: Self = const_ok(Self::new(b'b')).unwrap();
  /// Lowercase c
  pub const C_LOWER: Self = const_ok(Self::new(b'c')).unwrap();
  /// Lowercase d
  pub const D_LOWER: Self = const_ok(Self::new(b'd')).unwrap();
  /// Lowercase e
  pub const E_LOWER: Self = const_ok(Self::new(b'e')).unwrap();
  /// Lowercase f
  pub const F_LOWER: Self = const_ok(Self::new(b'f')).unwrap();
  /// Lowercase g
  pub const G_LOWER: Self = const_ok(Self::new(b'g')).unwrap();
  /// Lowercase h
  pub const H_LOWER: Self = const_ok(Self::new(b'h')).unwrap();
  /// Lowercase i
  pub const I_LOWER: Self = const_ok(Self::new(b'i')).unwrap();
  /// Lowercase j
  pub const J_LOWER: Self = const_ok(Self::new(b'j')).unwrap();
  /// Lowercase k
  pub const K_LOWER: Self = const_ok(Self::new(b'k')).unwrap();
  /// Lowercase l
  pub const L_LOWER: Self = const_ok(Self::new(b'l')).unwrap();
  /// Lowercase m
  pub const M_LOWER: Self = const_ok(Self::new(b'm')).unwrap();
  /// Lowercase n
  pub const N_LOWER: Self = const_ok(Self::new(b'n')).unwrap();
  /// Lowercase o
  pub const O_LOWER: Self = const_ok(Self::new(b'o')).unwrap();
  /// Lowercase p
  pub const P_LOWER: Self = const_ok(Self::new(b'p')).unwrap();
  /// Lowercase q
  pub const Q_LOWER: Self = const_ok(Self::new(b'q')).unwrap();
  /// Lowercase r
  pub const R_LOWER: Self = const_ok(Self::new(b'r')).unwrap();
  /// Lowercase s
  pub const S_LOWER: Self = const_ok(Self::new(b's')).unwrap();
  /// Lowercase t
  pub const T_LOWER: Self = const_ok(Self::new(b't')).unwrap();
  /// Lowercase u
  pub const U_LOWER: Self = const_ok(Self::new(b'u')).unwrap();
  /// Lowercase v
  pub const V_LOWER: Self = const_ok(Self::new(b'v')).unwrap();
  /// Lowercase w
  pub const W_LOWER: Self = const_ok(Self::new(b'w')).unwrap();
  /// Lowercase x
  pub const X_LOWER: Self = const_ok(Self::new(b'x')).unwrap();
  /// Lowercase y
  pub const Y_LOWER: Self = const_ok(Self::new(b'y')).unwrap();
  /// Lowercase z
  pub const Z_LOWER: Self = const_ok(Self::new(b'z')).unwrap();
  /// Left curly brace
  pub const LEFT_BRACE: Self = const_ok(Self::new(b'{')).unwrap();
  /// Pipe
  pub const PIPE: Self = const_ok(Self::new(b'|')).unwrap();
  /// Right curly brace
  pub const RIGHT_BRACE: Self = const_ok(Self::new(b'}')).unwrap();
  /// Tilde
  pub const TILDE: Self = const_ok(Self::new(b'~')).unwrap();
  /// Delete character
  pub const DELETE: Self = const_ok(Self::new(127)).unwrap();

  /// Generic constructor
  #[inline]
  pub const fn new(byte: u8) -> Result<Self, AsciiError> {
    match T::TY {
      AsciiTy::Alphabetic => {
        if byte.is_ascii_alphabetic() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotAlphabetic)
        }
      }
      AsciiTy::AlphaNumeric => {
        if byte.is_ascii_alphanumeric() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotAlphaNumeric)
        }
      }
      AsciiTy::Control => {
        if byte.is_ascii_control() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotControl)
        }
      }
      AsciiTy::Digit => {
        if byte.is_ascii_digit() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotDigit)
        }
      }
      AsciiTy::Graphic => {
        if byte.is_ascii_graphic() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotGraphic)
        }
      }
      AsciiTy::HexDigit => {
        if byte.is_ascii_hexdigit() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotHexDigit)
        }
      }
      AsciiTy::Lowercase => {
        if byte.is_ascii_lowercase() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotLowercase)
        }
      }
      AsciiTy::Punctuation => {
        if byte.is_ascii_punctuation() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotPunctuation)
        }
      }
      AsciiTy::Uppercase => {
        if byte.is_ascii_uppercase() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotUppercase)
        }
      }
      AsciiTy::Whitespace => {
        if byte.is_ascii_whitespace() {
          Ok(Self(byte, PhantomData))
        } else {
          Err(AsciiError::NotWhitespace)
        }
      }
    }
  }
}

impl<T> Ascii<T> {
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

impl<T> Clone for Ascii<T> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> Copy for Ascii<T> {}

impl<T> Display for Ascii<T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_char(self.as_char())
  }
}

impl<T> From<Ascii<T>> for char {
  #[inline]
  fn from(value: Ascii<T>) -> Self {
    value.as_char()
  }
}

impl<T> From<Ascii<T>> for u8 {
  #[inline]
  fn from(value: Ascii<T>) -> Self {
    value.0
  }
}

impl<T> FromStr for Ascii<T>
where
  T: AsciiParams,
{
  type Err = crate::Error;

  #[inline]
  fn from_str(value: &str) -> Result<Self, Self::Err> {
    value.as_bytes().try_into()
  }
}

impl<T, U> PartialEq<Ascii<U>> for Ascii<T> {
  #[inline]
  fn eq(&self, other: &Ascii<U>) -> bool {
    self.0 == other.0
  }
}

impl<T> PartialEq<u8> for Ascii<T> {
  #[inline]
  fn eq(&self, other: &u8) -> bool {
    self.0 == *other
  }
}

impl<T> PartialEq<char> for Ascii<T> {
  #[inline]
  fn eq(&self, other: &char) -> bool {
    self.as_char() == *other
  }
}

impl<T> TryFrom<&[u8]> for Ascii<T>
where
  T: AsciiParams,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let [byte] = value else {
      return Err(AsciiError::NonUnitSlice.into());
    };
    Ok(Self::new(*byte)?)
  }
}

impl<T> TryFrom<&str> for Ascii<T>
where
  T: AsciiParams,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    value.as_bytes().try_into()
  }
}

impl<T> TryFrom<char> for Ascii<T>
where
  T: AsciiParams,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: char) -> Result<Self, Self::Error> {
    Ok(Self::new(value.try_into().map_err(|_err| AsciiError::NonUnitChar)?)?)
  }
}

impl<T> TryFrom<u8> for Ascii<T>
where
  T: AsciiParams,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Ok(Self::new(value)?)
  }
}

/// Ascii Params
pub trait AsciiParams: private::Sealed {
  /// See [`AsciiTy`]
  const TY: AsciiTy;
}

macro_rules! create_params {
  (
    $(($name:ident, $ty:expr)),* $(,)?
  ) => {
    $(
      /// Parameters used by the associated ASCII range
      #[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
      pub struct $name;

      impl AsciiParams for $name {
        const TY: AsciiTy = $ty;
      }

      impl private::Sealed for $name {}
    )*
  };
}

macro_rules! impl_conversion {
  ($(($from:ty, $to:ty)),* $(,)?) => {
    $(
      impl From<$from> for $to {
        #[inline]
        fn from(value: $from) -> Self {
          Self(value.0, PhantomData)
        }
      }
    )*
  };
}

macro_rules! impl_default {
  (
    $(($name:ident, $char:expr)),* $(,)?
  ) => {
    $(
      impl Default for $name {
        #[inline]
        fn default() -> Self {
          Self($char, PhantomData)
        }
      }
    )*
  };
}

create_params! {
  (AsciiAlphabeticParams, AsciiTy::Alphabetic),
  (AsciiAlphaNumericParams, AsciiTy::AlphaNumeric),
  (AsciiControlParams, AsciiTy::Control),
  (AsciiDigitParams, AsciiTy::Digit),
  (AsciiGraphicParams, AsciiTy::Graphic),
  (AsciiHexDigitParams, AsciiTy::HexDigit),
  (AsciiLowercaseParams, AsciiTy::Lowercase),
  (AsciiPunctuationParams, AsciiTy::Punctuation),
  (AsciiUppercaseParams, AsciiTy::Uppercase),
  (AsciiWhitespaceParams, AsciiTy::Whitespace),
}

impl_conversion! {
  (AsciiAlphabetic, AsciiAlphaNumeric),
  (AsciiAlphabetic, AsciiGraphic),

  (AsciiAlphaNumeric, AsciiGraphic),

  (AsciiDigit, AsciiAlphaNumeric),
  (AsciiDigit, AsciiGraphic),
  (AsciiDigit, AsciiHexDigit),

  (AsciiHexDigit, AsciiAlphaNumeric),
  (AsciiHexDigit, AsciiGraphic),

  (AsciiLowercase, AsciiAlphabetic),
  (AsciiLowercase, AsciiAlphaNumeric),
  (AsciiLowercase, AsciiGraphic),

  (AsciiPunctuation, AsciiGraphic),

  (AsciiUppercase, AsciiAlphabetic),
  (AsciiUppercase, AsciiAlphaNumeric),
  (AsciiUppercase, AsciiGraphic),
}

impl_default! {
  (AsciiAlphabetic, b'a'),
  (AsciiAlphaNumeric, b'0'),
  (AsciiControl, 0),
  (AsciiDigit, b'0'),
  (AsciiGraphic, b'!'),
  (AsciiHexDigit, b'0'),
  (AsciiLowercase, b'a'),
  (AsciiPunctuation, b'.'),
  (AsciiUppercase, b'A'),
  (AsciiWhitespace, b' ')
}

mod private {
  #[allow(unnameable_types, reason = "intended behavior")]
  pub trait Sealed {}
}
