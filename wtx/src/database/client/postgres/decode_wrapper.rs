use crate::{
  database::client::{postgres::ty::Ty, rdbms::column_info::ColumnInfo},
  misc::Lease,
};

/// Struct used for decoding elements in PostgreSQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de, 'rem> {
  bytes: &'de [u8],
  name: &'rem str,
  ty: Ty,
}

impl<'de, 'rem> DecodeWrapper<'de, 'rem> {
  /// It is up to the caller to provide valid types to the decoding element.
  pub const fn new(bytes: &'de [u8], name: &'rem str, ty: Ty) -> Self {
    Self { bytes, name, ty }
  }

  /// Bytes of the column
  #[inline]
  pub const fn bytes(&self) -> &'de [u8] {
    self.bytes
  }

  /// Mutable version of [`Self::bytes`].
  #[inline]
  pub const fn bytes_mut(&mut self) -> &mut &'de [u8] {
    &mut self.bytes
  }

  /// Column's name
  #[inline]
  pub const fn name(&self) -> &'rem str {
    self.name
  }

  /// Mutable version of [`Self::name`].
  #[inline]
  pub const fn name_mut(&mut self) -> &mut &'rem str {
    &mut self.name
  }

  /// Type of the column.
  #[inline]
  pub const fn ty(&self) -> &Ty {
    &self.ty
  }

  /// Mutable version of [`Self::ty`].
  #[inline]
  pub const fn ty_mut(&mut self) -> &mut Ty {
    &mut self.ty
  }
}

impl Default for DecodeWrapper<'_, '_> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], name: "", ty: Ty::Any }
  }
}

impl Lease<[u8]> for DecodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

impl<'de, 'rem, C> From<(&'de [u8], &'rem C)> for DecodeWrapper<'de, 'rem>
where
  C: ColumnInfo<Ty = Ty>,
{
  #[inline]
  fn from(from: (&'de [u8], &'rem C)) -> Self {
    Self::new(from.0, from.1.name(), *from.1.ty())
  }
}
