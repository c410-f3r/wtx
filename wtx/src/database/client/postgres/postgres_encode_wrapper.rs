use crate::{
  collections::{TryExtend, VectorUsize},
  misc::{Lease, LeaseMut},
};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct PostgresEncodeWrapper<'bytes> {
  buffer: &'bytes mut VectorUsize<u8>,
}

impl<'bytes> PostgresEncodeWrapper<'bytes> {
  /// Shortcut
  #[inline]
  pub const fn new(buffer: &'bytes mut VectorUsize<u8>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to PostgreSQL.
  #[inline]
  pub const fn buffer(&mut self) -> &mut VectorUsize<u8> {
    self.buffer
  }
}

impl Lease<[u8]> for PostgresEncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer
  }
}

impl LeaseMut<[u8]> for PostgresEncodeWrapper<'_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [u8] {
    self.buffer
  }
}

impl<'slice> TryExtend<&'slice [u8]> for PostgresEncodeWrapper<'_> {
  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> crate::Result<()> {
    self.buffer.extend_from_copyable_slice(set)?;
    Ok(())
  }
}
