use crate::{
  database::{
    Identifier,
    client::{
      mysql::{mysql_protocol::column_res::ColumnRes, ty_params::TyParams},
      rdbms::column_info::ColumnInfo,
    },
  },
  misc::Lease,
};

#[derive(Clone, Debug)]
pub(crate) struct MysqlColumnInfo {
  pub(crate) name: Identifier,
  pub(crate) ty_params: TyParams,
}

impl MysqlColumnInfo {
  pub(crate) fn from_column_res(column_res: &ColumnRes) -> Self {
    let name = if column_res.alias.is_empty() { column_res.name } else { column_res.alias };
    Self { name, ty_params: TyParams::from_column_res(column_res) }
  }
}

impl ColumnInfo for MysqlColumnInfo {
  type Ty = TyParams;

  #[inline]
  fn name(&self) -> &str {
    &self.name
  }

  #[inline]
  fn ty(&self) -> &Self::Ty {
    &self.ty_params
  }
}

impl Lease<str> for MysqlColumnInfo {
  #[inline]
  fn lease(&self) -> &str {
    &self.name
  }
}

impl Lease<TyParams> for MysqlColumnInfo {
  #[inline]
  fn lease(&self) -> &TyParams {
    &self.ty_params
  }
}
