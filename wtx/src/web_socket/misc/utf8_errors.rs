use crate::web_socket::misc::IncompleteUtf8Char;

/// Extended error built upon [StdUtf8Error].
pub(crate) enum ExtUtf8Error {
  Incomplete { incomplete_ending_char: IncompleteUtf8Char },
  Invalid,
}

/// Standard error that is similar to the error type of the std.
pub(crate) struct StdUtf8Error {
  pub(crate) error_len: Option<usize>,
  pub(crate) valid_up_to: usize,
}
