use crate::{collection::Vector, misc::Lease};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct MysqlEncodeWrapper<'any> {
  buffer: &'any mut Vector<u8>,
}

impl<'any> MysqlEncodeWrapper<'any> {
  pub(crate) const fn new(buffer: &'any mut Vector<u8>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to MySQL.
  #[inline]
  pub const fn buffer(&mut self) -> &mut Vector<u8> {
    self.buffer
  }
}

impl Lease<[u8]> for MysqlEncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer
  }
}
