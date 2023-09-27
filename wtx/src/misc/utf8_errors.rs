use crate::misc::IncompleteUtf8Char;

pub(crate) enum ExtUtf8Error {
  Incomplete { incomplete_ending_char: IncompleteUtf8Char },
  Invalid,
}

pub(crate) struct StdUtf8Error {
  pub(crate) error_len: Option<usize>,
  pub(crate) valid_up_to: usize,
}
