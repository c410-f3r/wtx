use crate::misc::{FilledBufferWriter, Lease, LeaseMut};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct EncodeValue<'buffer, 'tmp> {
  fbw: &'tmp mut FilledBufferWriter<'buffer>,
}

impl<'buffer, 'tmp> EncodeValue<'buffer, 'tmp> {
  #[inline]
  pub(crate) fn new(fbw: &'tmp mut FilledBufferWriter<'buffer>) -> Self {
    Self { fbw }
  }

  /// See [`FilledBufferWriter`].
  #[inline]
  pub fn fbw(&mut self) -> &mut FilledBufferWriter<'buffer> {
    self.fbw
  }
}

impl<'buffer> Lease<FilledBufferWriter<'buffer>> for EncodeValue<'buffer, '_> {
  #[inline]
  fn lease(&self) -> &FilledBufferWriter<'buffer> {
    self.fbw
  }
}

impl<'buffer> LeaseMut<FilledBufferWriter<'buffer>> for EncodeValue<'buffer, '_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut FilledBufferWriter<'buffer> {
    self.fbw
  }
}
