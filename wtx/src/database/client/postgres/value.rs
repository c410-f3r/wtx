/// Implementation of [crate::database::DecodeInput] for PostgreSQL.
#[derive(Debug)]
pub struct Value<'bytes> {
  bytes: &'bytes [u8],
  is_null: bool,
}

impl<'bytes> Value<'bytes> {
  pub(crate) fn new(bytes: &'bytes [u8], is_null: bool) -> Self {
    Self { bytes, is_null }
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
    Self { bytes: &[], is_null: true }
  }
}

impl<'bytes> crate::database::Value for Value<'bytes> {
  #[inline]
  fn is_null(&self) -> bool {
    self.is_null
  }
}
