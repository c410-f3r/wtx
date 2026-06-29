use crate::{
  collections::{SuffixPusherVectorMut, TryExtend},
  misc::{Lease, LeaseMut},
};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct PostgresEncodeWrapper<'inner, 'outer> {
  buffer: &'outer mut SuffixPusherVectorMut<'inner, u8>,
}

impl<'inner, 'outer> PostgresEncodeWrapper<'inner, 'outer> {
  /// Shortcut
  #[inline]
  pub const fn new(buffer: &'outer mut SuffixPusherVectorMut<'inner, u8>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to PostgreSQL.
  #[inline]
  pub const fn buffer(&mut self) -> &mut SuffixPusherVectorMut<'inner, u8> {
    self.buffer
  }
}

impl Lease<[u8]> for PostgresEncodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer.curr()
  }
}

impl LeaseMut<[u8]> for PostgresEncodeWrapper<'_, '_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [u8] {
    self.buffer.curr_mut()
  }
}

impl<'slice> TryExtend<&'slice [u8]> for PostgresEncodeWrapper<'_, '_> {
  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> crate::Result<()> {
    self.buffer.inner_mut().extend_from_copyable_slice(set)?;
    Ok(())
  }
}
