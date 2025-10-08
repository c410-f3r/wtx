use crate::{
  database::{
    Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty},
  },
  de::{Decode, Encode},
};
use uuid::Uuid;

impl<'de, E> Decode<'de, Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut PostgresDecodeWrapper<'de, '_>) -> Result<Self, E> {
    let elem = Uuid::from_slice(dw.bytes()).map_err(Into::into)?;
    Ok(elem)
  }
}

impl<E> Encode<Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_bytes())?;
    Ok(())
  }
}

impl<E> Typed<Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Uuid)
  }
}

test!(uuid, Uuid, Uuid::max());
