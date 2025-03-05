use crate::{database::client::postgres::ty::Ty, misc::Lease};

/// Struct used for decoding elements in PostgreSQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  bytes: &'de [u8],
  ty: Ty,
}

impl<'de> DecodeWrapper<'de> {
  pub(crate) fn new(bytes: &'de [u8], ty: Ty) -> Self {
    Self { bytes, ty }
  }

  /// Bytes
  #[inline]
  pub fn bytes(&self) -> &'de [u8] {
    self.bytes
  }

  /// Type of a column.
  #[inline]
  pub fn ty(&self) -> &Ty {
    &self.ty
  }
}

impl Default for DecodeWrapper<'_> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], ty: Ty::Any }
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

impl<'de> From<(&'de [u8], Ty)> for DecodeWrapper<'de> {
  #[inline]
  fn from(from: (&'de [u8], Ty)) -> Self {
    Self::new(from.0, from.1)
  }
}
