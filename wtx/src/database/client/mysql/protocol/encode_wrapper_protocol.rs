use crate::{collection::Vector, misc::Lease};

pub(crate) struct EncodeWrapperProtocol<'any> {
  pub(crate) capabilities: &'any mut u64,
  pub(crate) encode_buffer: &'any mut Vector<u8>,
}

impl<'any> EncodeWrapperProtocol<'any> {
  pub(crate) const fn new(
    capabilities: &'any mut u64,
    encode_buffer: &'any mut Vector<u8>,
  ) -> Self {
    Self { capabilities, encode_buffer }
  }
}

impl Lease<[u8]> for EncodeWrapperProtocol<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.encode_buffer
  }
}
