use crate::{
  collections::{SuffixPusherVectorMut, TryExtend},
  misc::{Lease, LeaseMut},
};

/// Struct used for encoding TLS elements.
#[derive(Debug)]
pub(crate) struct TlsEncodeWrapper<'any> {
  buffer: SuffixPusherVectorMut<'any, u8>,
  is_hello_retry_request: bool,
}

impl<'any> TlsEncodeWrapper<'any> {
  pub(crate) const fn from_buffer(buffer: SuffixPusherVectorMut<'any, u8>) -> Self {
    Self { buffer, is_hello_retry_request: false }
  }

  #[inline]
  pub(crate) const fn buffer(&mut self) -> &mut SuffixPusherVectorMut<'any, u8> {
    &mut self.buffer
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request(&self) -> bool {
    self.is_hello_retry_request
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request_mut(&mut self) -> &mut bool {
    &mut self.is_hello_retry_request
  }
}

impl Lease<[u8]> for TlsEncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer.curr()
  }
}

impl LeaseMut<[u8]> for TlsEncodeWrapper<'_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [u8] {
    self.buffer.curr_mut()
  }
}

impl<'slice> TryExtend<&'slice [u8]> for TlsEncodeWrapper<'_> {
  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> crate::Result<()> {
    self.buffer.inner_mut().extend_from_copyable_slice(set)?;
    Ok(())
  }
}
