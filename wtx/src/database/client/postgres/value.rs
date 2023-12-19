/// Implementation of [crate::database::DecodeInput] for PostgreSQL.
#[derive(Debug)]
pub struct Value<'bytes> {
  bytes: &'bytes [u8],
}

impl<'bytes> Value<'bytes> {
  pub(crate) fn new(bytes: &'bytes [u8]) -> Self {
    Self { bytes }
  }

  /// Bytes
  #[inline]
  pub fn bytes(&self) -> &'bytes [u8] {
    self.bytes
  }
}

impl<'bytes> Default for Value<'bytes> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[] }
  }
}

impl<'bytes> crate::database::Value for Value<'bytes> {}
