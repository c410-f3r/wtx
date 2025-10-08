use crate::{
  database::client::{
    mysql::{Ty, TyParams},
    rdbms::column_info::ColumnInfo,
  },
  misc::Lease,
};

/// Struct used to represent decoded columns in MySQL.
#[derive(Debug, PartialEq)]
pub struct MysqlDecodeWrapper<'de, 'rem> {
  bytes: &'de [u8],
  name: &'rem str,
  ty: TyParams,
}

impl<'de, 'rem> MysqlDecodeWrapper<'de, 'rem> {
  pub(crate) const fn new(bytes: &'de [u8], name: &'rem str, ty: TyParams) -> Self {
    Self { bytes, name, ty }
  }

  /// Bytes of the column.
  #[inline]
  pub const fn bytes(&self) -> &'de [u8] {
    self.bytes
  }

  /// Column's name
  pub const fn name(&self) -> &'rem str {
    self.name
  }

  /// Type of the column.
  #[inline]
  pub const fn ty(&self) -> &TyParams {
    &self.ty
  }
}

impl Default for MysqlDecodeWrapper<'_, '_> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], name: "", ty: TyParams::empty(Ty::Null) }
  }
}

impl Lease<[u8]> for MysqlDecodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

impl<'de, 'rem, C> From<(&'de [u8], &'rem C)> for MysqlDecodeWrapper<'de, 'rem>
where
  C: ColumnInfo<Ty = TyParams>,
{
  #[inline]
  fn from(from: (&'de [u8], &'rem C)) -> Self {
    Self::new(from.0, from.1.name(), *from.1.ty())
  }
}
