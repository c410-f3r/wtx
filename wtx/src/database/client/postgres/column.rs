use crate::{
  database::{Identifier, client::postgres::Ty},
  misc::Lease,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Column {
  pub(crate) name: Identifier,
  pub(crate) ty: Ty,
}

impl Column {
  pub(crate) fn new(name: Identifier, ty: Ty) -> Self {
    Self { name, ty }
  }
}

impl Lease<str> for Column {
  #[inline]
  fn lease(&self) -> &str {
    &self.name
  }
}

impl Lease<Ty> for Column {
  #[inline]
  fn lease(&self) -> &Ty {
    &self.ty
  }
}
