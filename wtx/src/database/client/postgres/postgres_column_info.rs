use crate::{
  database::{
    Identifier,
    client::{postgres::Ty, rdbms::column_info::ColumnInfo},
  },
  misc::Lease,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostgresColumnInfo {
  pub(crate) name: Identifier,
  pub(crate) ty: Ty,
}

impl PostgresColumnInfo {
  pub(crate) const fn new(name: Identifier, ty: Ty) -> Self {
    Self { name, ty }
  }
}

impl ColumnInfo for PostgresColumnInfo {
  type Ty = Ty;

  #[inline]
  fn name(&self) -> &str {
    &self.name
  }

  #[inline]
  fn ty(&self) -> &Self::Ty {
    &self.ty
  }
}

impl Lease<str> for PostgresColumnInfo {
  #[inline]
  fn lease(&self) -> &str {
    &self.name
  }
}

impl Lease<Ty> for PostgresColumnInfo {
  #[inline]
  fn lease(&self) -> &Ty {
    &self.ty
  }
}
