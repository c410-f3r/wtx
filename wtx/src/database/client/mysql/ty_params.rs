use crate::database::client::mysql::{Ty, mysql_protocol::column_res::ColumnRes};

#[derive(Clone, Debug)]
pub(crate) struct TyParams {
  pub(crate) flags: u16,
  pub(crate) ty: Ty,
}

impl TyParams {
  #[inline]
  pub(crate) fn from_column_res(column_res: &ColumnRes) -> Self {
    Self { flags: column_res.flags, ty: column_res.ty }
  }
}
