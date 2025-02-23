use crate::database::{Identifier, client::postgres::Ty};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Column {
  pub(crate) name: Identifier,
  pub(crate) ty: Ty,
}

impl Column {
  #[inline]
  pub(crate) fn new(name: Identifier, ty: Ty) -> Self {
    Self { name, ty }
  }
}
