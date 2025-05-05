use crate::{
  database::client::mysql::{Ty, ty_params::TyParams},
  misc::Lease,
};

/// Struct used for decoding elements in MySQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  bytes: &'de [u8],
  ty: Ty,
}

impl<'de> DecodeWrapper<'de> {
  pub(crate) fn new(bytes: &'de [u8], ty: Ty) -> Self {
    Self { bytes, ty }
  }

  /// Bytes of a column.
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
    Self { bytes: &[], ty: Ty::Null }
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

impl<'de> From<(&'de [u8], TyParams)> for DecodeWrapper<'de> {
  #[inline]
  fn from(from: (&'de [u8], TyParams)) -> Self {
    Self::new(from.0, from.1.ty)
  }
}
