use crate::database::client::postgres::ty::Ty;

/// Struct used for decoding elements in PostgreSQL.
#[derive(Debug, PartialEq)]
pub struct DecodeValue<'any> {
  bytes: &'any [u8],
  ty: Ty,
}

impl<'any> DecodeValue<'any> {
  pub(crate) fn new(bytes: &'any [u8], ty: Ty) -> Self {
    Self { bytes, ty }
  }

  /// Bytes of a column.
  #[inline]
  pub fn bytes(&self) -> &'any [u8] {
    self.bytes
  }

  /// Type of a column.
  #[inline]
  pub fn ty(&self) -> &Ty {
    &self.ty
  }
}

impl Default for DecodeValue<'_> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], ty: Ty::Any }
  }
}
