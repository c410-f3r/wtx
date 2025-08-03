use crate::{misc::IncompleteUtf8Char, web_socket::OpCode};

#[derive(Debug)]
pub(crate) struct IsInContinuationFrame {
  pub(crate) iuc: Option<IncompleteUtf8Char>,
  pub(crate) op_code: OpCode,
  pub(crate) should_decompress: bool,
}
