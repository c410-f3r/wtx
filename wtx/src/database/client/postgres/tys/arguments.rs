use core::fmt::{Arguments, Write};

use crate::{
  database::{
    Typed,
    client::postgres::{EncodeWrapper, Postgres, Ty},
  },
  de::Encode,
};

impl<E> Encode<Postgres<E>> for Arguments<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().write_fmt(*self).map_err(crate::Error::from)?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for Arguments<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Text)
  }
}
