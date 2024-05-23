use crate::misc::IncompleteUtf8Char;

/// Extended error built upon [StdUtf8Error].
#[derive(Debug)]
pub enum ExtUtf8Error {
  /// More bytes are needed to validate the string.
  Incomplete {
    /// See [IncompleteUtf8Char].
    incomplete_ending_char: IncompleteUtf8Char,
  },
  /// It is impossible to validate the string
  Invalid,
}

/// Basic string error that doesn't contain any information.
#[derive(Debug)]
pub struct BasicUtf8Error;

impl From<BasicUtf8Error> for crate::Error {
  #[inline]
  fn from(_: BasicUtf8Error) -> Self {
    Self::MISC_InvalidUTF8
  }
}

/// Standard error that is similar to the error type of the standard library.
#[derive(Debug)]
pub struct StdUtf8Error {
  /// Error length
  pub error_len: Option<usize>,
  /// Starting index of mal-formatted bytes
  pub valid_up_to: usize,
}
