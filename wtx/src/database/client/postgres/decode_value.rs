use crate::database::client::postgres::ty::Ty;

/// Struct used for decoding elements in PostgreSQL.
#[derive(Debug, PartialEq)]
pub struct DecodeValue<'any> {
  bytes: &'any [u8],
  ty: &'any Ty,
}

impl<'any> DecodeValue<'any> {
  pub(crate) fn new(bytes: &'any [u8], ty: &'any Ty) -> Self {
    Self { bytes, ty }
  }

  /// Bytes of a column.
  #[inline]
  pub fn bytes(&self) -> &'any [u8] {
    self.bytes
  }

  /// Type of a column.
  #[inline]
  pub fn ty(&self) -> &'any Ty {
    self.ty
  }
}

impl<'any> Default for DecodeValue<'any> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], ty: &Ty::Any }
  }
}
