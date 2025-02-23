use crate::database::{
  Identifier,
  client::mysql::{mysql_protocol::column_res::ColumnRes, ty_params::TyParams},
};

#[derive(Clone, Debug)]
pub(crate) struct Column {
  pub(crate) name: Identifier,
  pub(crate) ty_params: TyParams,
}

impl Column {
  #[inline]
  pub(crate) fn from_column_res(column_res: &ColumnRes) -> Self {
    let name = if column_res.alias.is_empty() { column_res.name } else { column_res.alias };
    Self { name, ty_params: TyParams::from_column_res(column_res) }
  }
}
