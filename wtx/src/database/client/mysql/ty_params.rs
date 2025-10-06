use crate::database::client::mysql::{Ty, flag::Flag, mysql_protocol::column_res::ColumnRes};

/// [Ty] with metadata.
#[derive(Clone, Debug)]
pub struct TyParams {
  pub(crate) flags: u16,
  pub(crate) ty: Ty,
}

impl TyParams {
  pub(crate) const fn binary(ty: Ty) -> Self {
    Self { flags: Flag::Binary as u16, ty }
  }

  pub(crate) const fn empty(ty: Ty) -> Self {
    Self { flags: 0, ty }
  }

  pub(crate) const fn unsigned(ty: Ty) -> Self {
    Self { flags: Flag::Binary as u16 | Flag::Unsigned as u16, ty }
  }

  pub(crate) const fn from_column_res(column_res: &ColumnRes) -> Self {
    Self { flags: column_res.flags, ty: column_res.ty }
  }
}
