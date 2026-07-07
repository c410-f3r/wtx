use crate::{misc::PartialChar, web_socket::OpCode};

#[derive(Debug)]
pub(crate) struct IsInContinuationFrame {
  pub(crate) iuc: Option<PartialChar>,
  pub(crate) op_code: OpCode,
  pub(crate) should_decompress: bool,
}
