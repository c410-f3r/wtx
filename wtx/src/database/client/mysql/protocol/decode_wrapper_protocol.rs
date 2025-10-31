use crate::misc::Lease;

#[derive(Debug)]
pub(crate) struct DecodeWrapperProtocol<'inner, 'outer, O> {
  pub(crate) bytes: &'outer mut &'inner [u8],
  pub(crate) other: O,
}

impl<O> Lease<[u8]> for DecodeWrapperProtocol<'_, '_, O> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
