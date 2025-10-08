use crate::database::client::mysql::{Ty, flag::Flag, mysql_protocol::column_res::ColumnRes};

/// [`Ty`] with metadata.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyParams {
  flags: u16,
  ty: Ty,
}

impl TyParams {
  /// Constructor
  pub const fn new(flags: u16, ty: Ty) -> Self {
    Self { flags, ty }
  }

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

  /// Bitflag combination that compose a metadata.
  pub const fn flags(&self) -> u16 {
    self.flags
  }

  /// See [`Ty`].
  pub const fn ty(&self) -> Ty {
    self.ty
  }
}
